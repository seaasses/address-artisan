use crate::extended_public_key::ExtendedPubKey;
use hmac::{Hmac, Mac};
use ripemd::Ripemd160;
use secp256k1::{PublicKey, Secp256k1};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;

pub struct ExtendedPublicKeyDeriver {
    pub non_hardening_max_index: u32,
    derivation_cache: HashMap<Box<[u32]>, ExtendedPubKey>,
    secp: Secp256k1<secp256k1::All>,
    base_xpub: ExtendedPubKey,
}

const MAX_CACHE_SIZE: usize = 100_000;

impl ExtendedPublicKeyDeriver {
    pub fn new(base_xpub: &ExtendedPubKey) -> Self {
        Self {
            non_hardening_max_index: 0x7FFFFFFF,
            derivation_cache: HashMap::with_capacity(MAX_CACHE_SIZE),
            secp: Secp256k1::new(),
            base_xpub: base_xpub.clone(),
        }
    }

    fn derive_child(&self, parent: &ExtendedPubKey, index: u32) -> Result<ExtendedPubKey, String> {
        let mut data = Vec::with_capacity(37);
        data.extend_from_slice(&parent.public_key.serialize());
        data.extend_from_slice(&index.to_be_bytes());

        let mut hmac = Hmac::<Sha512>::new_from_slice(&parent.chain_code)
            .map_err(|e| format!("HMAC error: {}", e))?;
        hmac.update(&data);
        let result = hmac.finalize().into_bytes();

        let il = &result[0..32];
        let ir = &result[32..];

        let tweak =
            secp256k1::SecretKey::from_slice(il).map_err(|e| format!("Invalid tweak: {}", e))?;

        let child_pubkey = parent
            .public_key
            .combine(&PublicKey::from_secret_key(&self.secp, &tweak))
            .map_err(|e| format!("Failed to derive child key: {}", e))?;

        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(ir);

        Ok(ExtendedPubKey {
            public_key: child_pubkey,
            chain_code,
            depth: parent.depth + 1,
        })
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

    fn store_in_cache(&mut self, path: &[u32], xpub: ExtendedPubKey) {
        if self.derivation_cache.len() == MAX_CACHE_SIZE {
            self.derivation_cache.clear();
        }
        self.derivation_cache.insert(path.into(), xpub);
    }

    fn derive_single_step(
        &self,
        parent: &ExtendedPubKey,
        index: u32,
    ) -> Result<ExtendedPubKey, String> {
        if index > self.non_hardening_max_index {
            return Err(format!("{} is reserved for hardened derivation", index));
        }
        self.derive_child(parent, index)
    }

    fn get_derived_xpub(&mut self, path: &[u32]) -> Result<ExtendedPubKey, String> {
        if path.is_empty() {
            return Ok(self.base_xpub.clone());
        }

        let mut start_index = 0;
        let mut current_xpub = self.base_xpub.clone();

        for i in (0..path.len() - 1).rev() {
            let subpath = &path[0..=i];
            if let Some(cached) = self.get_from_cache(subpath) {
                current_xpub = cached.clone();
                start_index = i + 1;
                break;
            }
        }

        // Derive remaining steps
        for &index in path[start_index..].iter() {
            current_xpub = self.derive_single_step(&current_xpub, index)?;

            // Cache intermediate paths
            if start_index < path.len() - 1 {
                self.store_in_cache(&path[0..=start_index], current_xpub.clone());
            }
            start_index += 1;
        }

        Ok(current_xpub)
    }
}
