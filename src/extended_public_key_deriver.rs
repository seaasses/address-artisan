use bs58;
use hmac;
use ripemd::Ripemd160;
use secp256k1::{PublicKey, Secp256k1};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::{mpsc, Arc, RwLock};
use threadpool::ThreadPool;

const MAX_CACHE_SIZE: usize = 1_000_000;

#[derive(Clone, Debug)]
pub struct ExtendedPubKey {
    pub public_key: PublicKey,
    pub chain_code: [u8; 32],
    pub depth: u8,
}

impl ExtendedPubKey {
    pub fn from_str(xpub: &str) -> Result<Self, String> {
        if !xpub.starts_with("xpub") {
            return Err("Invalid xpub format: must start with 'xpub'".to_string());
        }

        let data = Self::decode_base58(xpub)?;
        let (payload, checksum) = data.split_at(78);
        Self::verify_checksum(payload, checksum)?;

        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(&payload[13..45]);

        let public_key = PublicKey::from_slice(&payload[45..78])
            .map_err(|e| format!("Invalid public key: {}", e))?;

        Ok(ExtendedPubKey {
            public_key,
            chain_code,
            depth: payload[4],
        })
    }

    fn decode_base58(xpub: &str) -> Result<Vec<u8>, String> {
        let data = bs58::decode(xpub)
            .into_vec()
            .map_err(|e| format!("Failed to decode base58: {}", e))?;

        if data.len() != 82 {
            return Err(format!("Invalid xpub length: {}", data.len()));
        }

        Ok(data)
    }

    fn verify_checksum(payload: &[u8], checksum: &[u8]) -> Result<(), String> {
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
        Ok(())
    }

    pub fn derive_child(
        &self,
        secp: &Secp256k1<secp256k1::All>,
        index: u32,
    ) -> Result<Self, String> {
        let (il, ir) = self.generate_child_keys(index)?;
        let child_pubkey = self.compute_child_pubkey(secp, &il)?;

        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(ir.as_slice());

        Ok(ExtendedPubKey {
            public_key: child_pubkey,
            chain_code,
            depth: self.depth + 1,
        })
    }

    fn generate_child_keys(&self, index: u32) -> Result<(Vec<u8>, Vec<u8>), String> {
        use hmac::{Hmac, Mac};
        type HmacSha512 = Hmac<sha2::Sha512>;

        let mut data = Vec::with_capacity(37);
        data.extend_from_slice(&self.public_key.serialize());
        data.extend_from_slice(&index.to_be_bytes());

        let mut hmac = HmacSha512::new_from_slice(&self.chain_code)
            .map_err(|e| format!("HMAC error: {}", e))?;
        hmac.update(&data);
        let result = hmac.finalize().into_bytes();

        Ok((result[0..32].to_vec(), result[32..].to_vec()))
    }

    fn compute_child_pubkey(
        &self,
        secp: &Secp256k1<secp256k1::All>,
        il: &[u8],
    ) -> Result<PublicKey, String> {
        let tweak =
            secp256k1::SecretKey::from_slice(il).map_err(|e| format!("Invalid tweak: {}", e))?;

        self.public_key
            .combine(&PublicKey::from_secret_key(secp, &tweak))
            .map_err(|e| format!("Failed to derive child key: {}", e))
    }
}

// Core derivation logic that works only with cache
pub struct ExtendedPubKeyCore {
    base_xpub: ExtendedPubKey,
    derivation_cache: Arc<RwLock<HashMap<Vec<u32>, ExtendedPubKey>>>,
    secp: Secp256k1<secp256k1::All>,
    non_hardening_max_index: u32,
}

impl ExtendedPubKeyCore {
    pub fn new(
        base_xpub: ExtendedPubKey,
        cache: Arc<RwLock<HashMap<Vec<u32>, ExtendedPubKey>>>,
    ) -> Self {
        Self {
            base_xpub,
            derivation_cache: cache,
            secp: Secp256k1::new(),
            non_hardening_max_index: 0x7FFFFFFF,
        }
    }

    pub fn get_pubkey_hash_160(&self, path: &[u32]) -> Result<[u8; 20], String> {
        let pubkey = self.get_pubkey(path)?;
        self.compute_hash160(&pubkey)
    }

    pub fn get_pubkey(&self, path: &[u32]) -> Result<[u8; 33], String> {
        let derived_xpub = self.get_derived_xpub(path)?;
        Ok(derived_xpub.public_key.serialize())
    }

    fn compute_hash160(&self, pubkey: &[u8]) -> Result<[u8; 20], String> {
        let mut hasher = Sha256::new();
        hasher.update(pubkey);
        let hash = hasher.finalize();

        let mut ripemd_hasher = Ripemd160::new();
        ripemd_hasher.update(hash);
        let hash = ripemd_hasher.finalize();

        let mut result = [0u8; 20];
        result.copy_from_slice(&hash);
        Ok(result)
    }

    fn get_from_cache(&self, path: &[u32]) -> Option<ExtendedPubKey> {
        self.derivation_cache.read().ok()?.get(path).cloned()
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
            return Ok(self.base_xpub.clone());
        }

        if let Some(cached) = self.get_from_cache(path) {
            return Ok(cached);
        }

        self.derive_path(path)
    }

    fn derive_path(&self, path: &[u32]) -> Result<ExtendedPubKey, String> {
        let mut current_path = Vec::with_capacity(path.len());
        let mut current_xpub = self.base_xpub.clone();

        for (i, &index) in path.iter().enumerate() {
            current_path.push(index);

            if i < path.len() - 1 {
                if let Some(cached) = self.get_from_cache(&current_path) {
                    current_xpub = cached;
                    continue;
                }
            }

            current_xpub = self.derive_single_step(&current_xpub, index)?;
        }

        Ok(current_xpub)
    }
}

// Orchestrator that handles threading and cache management
pub struct ExtendedPublicKeyDeriver {
    derivation_cache: Arc<RwLock<HashMap<Vec<u32>, ExtendedPubKey>>>,
    thread_pool: ThreadPool,
    base_xpub: ExtendedPubKey,
}

impl ExtendedPublicKeyDeriver {
    pub fn new(xpub: &str) -> Result<Self, String> {
        let base_xpub = ExtendedPubKey::from_str(xpub)?;
        let num_threads = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);

        Ok(Self {
            derivation_cache: Arc::new(RwLock::new(HashMap::new())),
            thread_pool: ThreadPool::new(num_threads),
            base_xpub,
        })
    }

    pub fn get_pubkeys_hash_160(&self, paths: &[Vec<u32>]) -> Result<Vec<[u8; 20]>, String> {
        let paths_to_cache = self.collect_ancestor_paths(paths)?;
        self.populate_cache(&paths_to_cache)?;
        self.process_paths_in_parallel(paths)
    }

    fn collect_ancestor_paths(&self, paths: &[Vec<u32>]) -> Result<HashSet<Vec<u32>>, String> {
        let mut paths_to_cache = HashSet::new();
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

        if paths_to_cache.len() > MAX_CACHE_SIZE {
            println!("need to cache {} paths", paths_to_cache.len());
            panic!("Cache size exceeded");
        }

        Ok(paths_to_cache)
    }

    fn populate_cache(&self, paths_to_cache: &HashSet<Vec<u32>>) -> Result<(), String> {
        let core = ExtendedPubKeyCore::new(self.base_xpub.clone(), self.derivation_cache.clone());
        let mut ordered_paths: Vec<_> = paths_to_cache.iter().collect();
        ordered_paths.sort_by_key(|p| p.len());

        for path in ordered_paths {
            let derived = core.get_derived_xpub(path)?;
            if let Ok(mut cache) = self.derivation_cache.write() {
                cache.insert(path.clone(), derived);
            }
        }

        Ok(())
    }

    fn process_paths_in_parallel(&self, paths: &[Vec<u32>]) -> Result<Vec<[u8; 20]>, String> {
        let chunk_size =
            (paths.len() + self.thread_pool.max_count() - 1) / self.thread_pool.max_count();
        let (tx, rx) = mpsc::channel();

        self.spawn_worker_threads(paths, chunk_size, tx);
        self.collect_results(paths.len(), rx)
    }

    fn spawn_worker_threads(
        &self,
        paths: &[Vec<u32>],
        chunk_size: usize,
        tx: mpsc::Sender<Vec<Result<[u8; 20], String>>>,
    ) {
        for chunk in paths.chunks(chunk_size) {
            let chunk_paths = chunk.to_vec();
            let cache = self.derivation_cache.clone();
            let base_xpub = self.base_xpub.clone();
            let tx = tx.clone();

            self.thread_pool.execute(move || {
                let core = ExtendedPubKeyCore::new(base_xpub, cache);
                let mut results = Vec::with_capacity(chunk_paths.len());

                for path in chunk_paths {
                    match core.get_pubkey_hash_160(&path) {
                        Ok(hash) => results.push(Ok(hash)),
                        Err(e) => results.push(Err(e)),
                    }
                }
                tx.send(results).unwrap();
            });
        }
    }

    fn collect_results(
        &self,
        total_paths: usize,
        rx: mpsc::Receiver<Vec<Result<[u8; 20], String>>>,
    ) -> Result<Vec<[u8; 20]>, String> {
        let mut all_results = Vec::with_capacity(total_paths);

        while let Ok(chunk_results) = rx.recv() {
            for result in chunk_results {
                all_results.push(result?);
            }
        }

        Ok(all_results)
    }
}

impl Drop for ExtendedPublicKeyDeriver {
    fn drop(&mut self) {
        if let Ok(mut cache) = self.derivation_cache.write() {
            cache.clear();
        }
    }
}
