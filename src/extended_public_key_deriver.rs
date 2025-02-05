use bs58;
use hmac;
use ripemd::Ripemd160;
use secp256k1::{PublicKey, Secp256k1};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::{mpsc, Arc, RwLock};
use threadpool::ThreadPool;

#[derive(Clone, Debug)]
pub struct ExtendedPubKey {
    pub public_key: PublicKey,
    pub chain_code: [u8; 32],
    pub depth: u8,
}

pub struct ExtendedPublicKeyDeriver {
    xpub: String,
    pub non_hardening_max_index: u32,
    derivation_cache: Arc<RwLock<HashMap<Vec<u32>, ExtendedPubKey>>>,
    secp: Secp256k1<secp256k1::All>,
    base_xpub: Option<ExtendedPubKey>,
    thread_pool: ThreadPool,
}

impl Drop for ExtendedPublicKeyDeriver {
    fn drop(&mut self) {
        if let Ok(mut cache) = self.derivation_cache.write() {
            cache.clear();
        }
    }
}

const MAX_CACHE_SIZE: usize = 100_00000;

impl ExtendedPubKey {
    pub fn from_str(xpub: &str) -> Result<Self, String> {
        if !xpub.starts_with("xpub") {
            return Err("Invalid xpub format: must start with 'xpub'".to_string());
        }

        let data = bs58::decode(xpub)
            .into_vec()
            .map_err(|e| format!("Failed to decode base58: {}", e))?;

        if data.len() != 82 {
            return Err(format!("Invalid xpub length: {}", data.len()));
        }

        let payload = &data[0..78];
        let checksum = &data[78..82];

        let mut hasher = Sha256::new();
        hasher.update(payload);
        let first_hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(&first_hash);
        let second_hash = hasher.finalize();

        if checksum != &second_hash[0..4] {
            return Err(format!(
                "Invalid checksum: expected {:02x?}, got {:02x?}",
                checksum,
                &second_hash[0..4]
            ));
        }

        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(&payload[13..45]);

        let public_key = PublicKey::from_slice(&payload[45..78])
            .map_err(|e| format!("Invalid public key: {}", e))?;

        let mut parent_fingerprint = [0u8; 4];
        parent_fingerprint.copy_from_slice(&payload[5..9]);

        Ok(ExtendedPubKey {
            public_key,
            chain_code,
            depth: payload[4],
        })
    }

    pub fn derive_child(
        &self,
        secp: &Secp256k1<secp256k1::All>,
        index: u32,
    ) -> Result<Self, String> {
        use hmac::{Hmac, Mac};
        type HmacSha512 = Hmac<sha2::Sha512>;

        let mut data = Vec::with_capacity(37);
        data.extend_from_slice(&self.public_key.serialize());
        data.extend_from_slice(&index.to_be_bytes());

        let mut hmac = HmacSha512::new_from_slice(&self.chain_code)
            .map_err(|e| format!("HMAC error: {}", e))?;
        hmac.update(&data);
        let result = hmac.finalize().into_bytes();

        let il = &result[0..32];
        let ir = &result[32..];

        let tweak =
            secp256k1::SecretKey::from_slice(il).map_err(|e| format!("Invalid tweak: {}", e))?;

        let child_pubkey = self
            .public_key
            .combine(&PublicKey::from_secret_key(secp, &tweak))
            .map_err(|e| format!("Failed to derive child key: {}", e))?;

        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(ir);

        Ok(ExtendedPubKey {
            public_key: child_pubkey,
            chain_code,
            depth: self.depth + 1,
        })
    }
}

impl ExtendedPublicKeyDeriver {
    pub fn new(xpub: &str) -> Self {
        let base = ExtendedPubKey::from_str(xpub).ok();

        let num_threads = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);

        Self {
            xpub: xpub.to_string(),
            non_hardening_max_index: 0x7FFFFFFF,
            derivation_cache: Arc::new(RwLock::new(HashMap::new())),
            secp: Secp256k1::new(),
            base_xpub: base,
            thread_pool: ThreadPool::new(num_threads),
        }
    }

    pub fn get_pubkeys_hash_160(&self, paths: &[Vec<u32>]) -> Result<Vec<[u8; 20]>, String> {
        let mut paths_to_cache: HashSet<Vec<u32>> = HashSet::new();
        {
            let cache_read = self.derivation_cache.read().map_err(|e| e.to_string())?;
            for path in paths {
                if path.len() > 1 {
                    for i in 1..path.len() {
                        let ancestor = path[0..i].to_vec();
                        if !cache_read.contains_key(&ancestor) {
                            paths_to_cache.insert(ancestor);
                        }
                    }
                }
            }
            if cache_read.len() + paths_to_cache.len() >= MAX_CACHE_SIZE {
                drop(cache_read);
                self.derivation_cache
                    .write()
                    .map_err(|e| e.to_string())?
                    .clear();
            }
        }

        if paths_to_cache.len() > MAX_CACHE_SIZE / 10 {
            println!("need to cache {} paths", paths_to_cache.len());
            panic!("Cache size exceeded");
        }

        let mut ordered_paths: Vec<_> = paths_to_cache.into_iter().collect();
        ordered_paths.sort_by_key(|p| p.len());
        for path_to_cache in ordered_paths {
            let derived = self.get_derived_xpub(&path_to_cache)?;
            self.derivation_cache
                .write()
                .map_err(|e| e.to_string())?
                .insert(path_to_cache, derived);
        }

        let chunk_size =
            (paths.len() + self.thread_pool.max_count() - 1) / self.thread_pool.max_count();
        let (tx, rx) = mpsc::channel();

        for chunk in paths.chunks(chunk_size) {
            let chunk_paths = chunk.to_vec();
            let xpub = self.xpub.clone();
            let base_xpub = self.base_xpub.clone();
            let tx = tx.clone();
            let non_hardening_max_index = self.non_hardening_max_index;

            self.thread_pool.execute(move || {
                let mut results = Vec::with_capacity(chunk_paths.len());
                let mut deriver = ExtendedPublicKeyDeriver {
                    xpub,
                    non_hardening_max_index,
                    derivation_cache: Arc::new(RwLock::new(HashMap::new())), // Thread-local cache
                    secp: Secp256k1::new(),
                    base_xpub,
                    thread_pool: ThreadPool::new(1), // Dummy pool for thread-local instance
                };

                for path in chunk_paths {
                    match deriver.get_pubkey_hash_160(&path) {
                        Ok(hash) => results.push(Ok(hash)),
                        Err(e) => results.push(Err(e)),
                    }
                }
                tx.send(results).unwrap();
            });
        }

        drop(tx);

        let mut all_results = Vec::with_capacity(paths.len());
        while let Ok(chunk_results) = rx.recv() {
            for result in chunk_results {
                all_results.push(result?);
            }
        }

        Ok(all_results)
    }

    pub fn get_pubkey_hash_160(&mut self, path: &[u32]) -> Result<[u8; 20], String> {
        let pubkey = self.get_pubkey(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&pubkey);
        let hash = hasher.finalize();

        let mut ripemd_hasher = Ripemd160::new();
        ripemd_hasher.update(hash);
        let hash = ripemd_hasher.finalize();

        let mut result = [0u8; 20];
        result.copy_from_slice(&hash);
        Ok(result)
    }

    pub fn get_pubkey(&mut self, path: &[u32]) -> Result<[u8; 33], String> {
        let derived_xpub = self.get_derived_xpub(path)?;
        Ok(derived_xpub.public_key.serialize())
    }

    fn get_from_cache(&self, path: &[u32]) -> Option<ExtendedPubKey> {
        self.derivation_cache.read().ok()?.get(path).cloned()
    }

    fn store_in_cache(&self, path: Vec<u32>, xpub: ExtendedPubKey) {
        if let Ok(mut cache) = self.derivation_cache.write() {
            if cache.len() >= MAX_CACHE_SIZE {
                cache.clear();
            }
            cache.insert(path, xpub);
        }
    }

    fn get_base_xpub(&self) -> Result<ExtendedPubKey, String> {
        if let Some(ref base) = self.base_xpub {
            return Ok(base.clone());
        }
        ExtendedPubKey::from_str(&self.xpub)
    }

    fn derive_single_step(
        &self,
        parent: &ExtendedPubKey,
        index: u32,
    ) -> Result<ExtendedPubKey, String> {
        if index > self.non_hardening_max_index {
            return Err(format!("{} is reserved for hardened derivation", index));
        }
        parent.derive_child(&self.secp, index)
    }

    fn get_derived_xpub(&self, path: &[u32]) -> Result<ExtendedPubKey, String> {
        if path.is_empty() {
            return self.get_base_xpub();
        }

        // Try to get from cache using the full path
        if let Some(cached) = self.get_from_cache(path) {
            return Ok(cached.clone());
        }

        let mut current_path = Vec::with_capacity(path.len());
        let mut current_xpub = self.get_base_xpub()?;

        for (i, &index) in path.iter().enumerate() {
            current_path.push(index);

            // Only check cache for non-final paths to avoid unnecessary lookups
            if i < path.len() - 1 {
                if let Some(cached) = self.get_from_cache(&current_path) {
                    current_xpub = cached.clone();
                    continue;
                }
            }

            current_xpub = self.derive_single_step(&current_xpub, index)?;

            // Cache all intermediate paths
            if i < path.len() - 1 {
                self.store_in_cache(current_path.clone(), current_xpub.clone());
            }
        }

        Ok(current_xpub)
    }
}
