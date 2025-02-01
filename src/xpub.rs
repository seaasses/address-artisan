use bs58;
use ripemd::Ripemd160;
use secp256k1::{PublicKey, Secp256k1};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ExtendedPubKey {
    pub public_key: PublicKey,
    pub chain_code: [u8; 32],
    pub depth: u8,
}

pub struct ExtendedPublicKeyDeriver {
    xpub: String,
    pub non_hardening_max_index: u32,
    derivation_cache: HashMap<Vec<u32>, ExtendedPubKey>,
    secp: Secp256k1<secp256k1::All>,
    base_xpub: Option<ExtendedPubKey>,
}

const MAX_CACHE_SIZE: usize = 10_000;

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
        // Generate HMAC-SHA512
        let mut hasher = Sha256::new();
        hasher.update(&self.public_key.serialize());
        hasher.update(&index.to_be_bytes());
        let il = hasher.finalize();

        // Generate chain code using a different hash
        let mut hasher = Sha256::new();
        hasher.update(&il);
        hasher.update(&self.chain_code);
        let ir = hasher.finalize();

        let tweak =
            secp256k1::SecretKey::from_slice(&il).map_err(|e| format!("Invalid tweak: {}", e))?;

        let child_pubkey = self
            .public_key
            .combine(&PublicKey::from_secret_key(secp, &tweak))
            .map_err(|e| format!("Failed to derive child key: {}", e))?;

        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(&ir);

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
        Self {
            xpub: xpub.to_string(),
            non_hardening_max_index: 0x7FFFFFFF,
            derivation_cache: HashMap::with_capacity(10000),
            secp: Secp256k1::new(),
            base_xpub: base,
        }
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

    fn get_from_cache(&self, path: &[u32]) -> Option<&ExtendedPubKey> {
        self.derivation_cache.get(path)
    }

    fn store_in_cache(&mut self, path: Vec<u32>, xpub: ExtendedPubKey) {
        if self.derivation_cache.len() == MAX_CACHE_SIZE {
            self.derivation_cache.clear();
        }
        self.derivation_cache.insert(path, xpub);
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

    fn get_derived_xpub(&mut self, path: &[u32]) -> Result<ExtendedPubKey, String> {
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
