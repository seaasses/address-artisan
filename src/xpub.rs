use bitcoin::bip32::{ChildNumber, Xpub};
use bitcoin::hashes::{ripemd160, sha256, Hash};
use secp256k1::Secp256k1;
use std::collections::HashMap;
use std::str::FromStr;

pub struct XpubWrapper {
    xpub: String,
    pub non_hardening_max_index: u32,
    derivation_cache: HashMap<Vec<u32>, Xpub>,
    secp: Secp256k1<secp256k1::All>,
}

impl XpubWrapper {
    pub fn new(xpub: &str) -> Self {
        Self {
            xpub: xpub.to_string(),
            non_hardening_max_index: 0x7FFFFFFF,
            derivation_cache: HashMap::new(),
            secp: Secp256k1::new(),
        }
    }

    pub fn get_pubkey_hash_160(&mut self, path: Vec<u32>) -> Result<[u8; 20], String> {
        let pubkey = self.get_pubkey(path)?;
        let pubkey_hash_sha256 = sha256::Hash::hash(&pubkey);
        let pubkey_hash_ripemd160 = ripemd160::Hash::hash(pubkey_hash_sha256.as_ref());
        Ok(pubkey_hash_ripemd160.to_byte_array())
    }

    pub fn get_pubkey(&mut self, path: Vec<u32>) -> Result<[u8; 33], String> {
        let derived_xpub = self.get_derived_xpub(path)?;
        Ok(derived_xpub.public_key.serialize())
    }

    fn get_from_cache(&self, path: &[u32]) -> Option<&Xpub> {
        self.derivation_cache.get(path)
    }

    fn store_in_cache(&mut self, path: Vec<u32>, xpub: Xpub) {
        self.derivation_cache.insert(path, xpub);
    }

    fn get_base_xpub(&self) -> Result<Xpub, String> {
        Xpub::from_str(&self.xpub).map_err(|e| format!("Failed to parse xpub: {}", e))
    }

    fn derive_single_step(&self, parent: &Xpub, index: u32) -> Result<Xpub, String> {
        if index > self.non_hardening_max_index {
            return Err(format!("{} is reserved for hardened derivation", index));
        }
        let child_number = ChildNumber::from_normal_idx(index)
            .map_err(|_| format!("Invalid child number: {}", index))?;

        parent
            .derive_pub(&self.secp, &[child_number])
            .map_err(|e| e.to_string())
    }

    fn get_derived_xpub(&mut self, path: Vec<u32>) -> Result<Xpub, String> {
        if path.is_empty() {
            return self.get_base_xpub();
        }

        // Try to get from cache first
        if let Some(cached) = self.get_from_cache(&path) {
            return Ok(cached.clone());
        }

        // Find the longest cached parent path
        let mut current_path = Vec::new();
        let mut current_xpub = self.get_base_xpub()?;

        for &index in path.iter() {
            current_path.push(index);

            // Try to get this level from cache
            if let Some(cached) = self.get_from_cache(&current_path) {
                current_xpub = cached.clone();
                continue;
            }

            // Not in cache, need to derive and store
            current_xpub = self.derive_single_step(&current_xpub, index)?;
            self.store_in_cache(current_path.clone(), current_xpub.clone());
        }

        Ok(current_xpub)
    }
}
