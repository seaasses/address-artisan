use crate::constants::NON_HARDENED_MAX_INDEX;
use crate::extended_public_key::ExtendedPubKey;
use hmac::{Hmac, Mac};
use lru::LruCache;
use ripemd::Ripemd160;
use secp256k1::{PublicKey, Secp256k1};
use sha2::{Digest, Sha256, Sha512};
use std::num::NonZeroUsize;

type ExtendedKeyResult = Result<([u8; 32], [u8; 32], [u8; 32]), String>;

pub trait KeyDeriver {
    fn get_pubkey_hash_160(&mut self, path: &[u32]) -> Result<[u8; 20], String>;
    fn get_pubkey(&mut self, path: &[u32]) -> Result<[u8; 33], String>;
}

pub struct ExtendedPublicKeyDeriver {
    derivation_cache: LruCache<Box<[u32]>, ExtendedPubKey>,
    secp: Secp256k1<secp256k1::All>,
    base_xpub: ExtendedPubKey,
    sha256_hasher: Sha256,
    ripemd_hasher: Ripemd160,
}

const MAX_CACHE_SIZE: usize = 10_000;

impl ExtendedPublicKeyDeriver {
    pub fn new(base_xpub: &ExtendedPubKey) -> Self {
        Self {
            derivation_cache: LruCache::new(NonZeroUsize::new(MAX_CACHE_SIZE).unwrap()),
            secp: Secp256k1::new(),
            base_xpub: base_xpub.clone(),
            sha256_hasher: Sha256::new(),
            ripemd_hasher: Ripemd160::new(),
        }
    }

    fn derive_child(&self, parent: &ExtendedPubKey, index: u32) -> Result<ExtendedPubKey, String> {
        let mut data = [0u8; 37]; // 33 bytes pubkey + 4 bytes index
        data[0..33].copy_from_slice(&parent.public_key.serialize());
        data[33..37].copy_from_slice(&index.to_be_bytes());

        let mut hmac = Hmac::<Sha512>::new_from_slice(&parent.chain_code)
            .map_err(|e| format!("HMAC error: {}", e))?;
        hmac.update(&data);
        let result = hmac.finalize().into_bytes();

        let il = &result[0..32];
        let ir = &result[32..];

        let mut il_array = [0u8; 32];
        il_array.copy_from_slice(il);
        let tweak = secp256k1::SecretKey::from_byte_array(il_array)
            .map_err(|e| format!("Invalid tweak: {}", e))?;

        let child_pubkey = parent
            .public_key
            .combine(&PublicKey::from_secret_key(&self.secp, &tweak))
            .map_err(|e| format!("Failed to derive child key: {}", e))?;

        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(ir);

        Ok(ExtendedPubKey {
            public_key: child_pubkey,
            chain_code,
        })
    }

    fn derive_single_step(
        &self,
        parent: &ExtendedPubKey,
        index: u32,
    ) -> Result<ExtendedPubKey, String> {
        if index > NON_HARDENED_MAX_INDEX {
            return Err(format!("{} is reserved for hardened derivation", index));
        }
        self.derive_child(parent, index)
    }
}

impl KeyDeriver for ExtendedPublicKeyDeriver {
    fn get_pubkey_hash_160(&mut self, path: &[u32]) -> Result<[u8; 20], String> {
        let pubkey = self.get_pubkey(path)?;

        self.sha256_hasher.reset();
        self.sha256_hasher.update(pubkey);
        let hash = self.sha256_hasher.finalize_reset();

        self.ripemd_hasher.reset();
        self.ripemd_hasher.update(hash);
        let hash = self.ripemd_hasher.finalize_reset();

        let mut result = [0u8; 20];
        result.copy_from_slice(&hash);
        Ok(result)
    }

    fn get_pubkey(&mut self, path: &[u32]) -> Result<[u8; 33], String> {
        let derived_xpub = self.get_derived_xpub(path)?;
        Ok(derived_xpub.public_key.serialize())
    }
}

impl ExtendedPublicKeyDeriver {
    fn get_derived_xpub(&mut self, path: &[u32]) -> Result<ExtendedPubKey, String> {
        if path.is_empty() {
            return Ok(self.base_xpub.clone());
        }

        let mut start_index = 0;
        let mut current_xpub = None;

        for i in (0..path.len() - 1).rev() {
            let subpath = &path[0..=i];
            if let Some(cached) = self.derivation_cache.get(subpath) {
                current_xpub = Some(cached.clone());
                start_index = i + 1;
                break;
            }
        }

        let mut current_xpub = current_xpub.unwrap_or_else(|| self.base_xpub.clone());

        for (offset, &index) in path[start_index..].iter().enumerate() {
            current_xpub = self.derive_single_step(&current_xpub, index)?;
            let current_depth = start_index + offset;
            if current_depth < path.len() - 1 {
                let cache_path = &path[0..=current_depth];
                self.derivation_cache
                    .put(cache_path.into(), current_xpub.clone());
            }
        }

        Ok(current_xpub)
    }

    pub fn get_extended_key(&mut self, path: &[u32]) -> ExtendedKeyResult {
        let xpub = self.get_derived_xpub(path)?;

        let uncompressed = xpub.public_key.serialize_uncompressed();

        let mut x_coord = [0u8; 32];
        let mut y_coord = [0u8; 32];
        x_coord.copy_from_slice(&uncompressed[1..33]);
        y_coord.copy_from_slice(&uncompressed[33..65]);

        Ok((xpub.chain_code, x_coord, y_coord))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_extended_key_returns_valid_data() {
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);

        let path = [131, 1342, 0, 0, 0];
        let (chain_code, x_coord, y_coord) = deriver.get_extended_key(&path).unwrap();

        assert_eq!(chain_code.len(), 32);
        assert_eq!(x_coord.len(), 32);
        assert_eq!(y_coord.len(), 32);

        assert_ne!(chain_code, [0u8; 32]);
        assert_ne!(x_coord, [0u8; 32]);
        assert_ne!(y_coord, [0u8; 32]);
    }

    #[test]
    fn test_get_extended_key_matches_get_pubkey() {
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);

        let path = [131, 1342, 0, 0, 0];

        let compressed_key = deriver.get_pubkey(&path).unwrap();
        let (_chain_code, x_coord, y_coord) = deriver.get_extended_key(&path).unwrap();

        assert_eq!(compressed_key[0], 0x02 | (y_coord[31] & 1));
        assert_eq!(&compressed_key[1..33], &x_coord[..]);
    }

    #[test]
    fn test_get_extended_key_same_path_same_result() {
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);

        let path = [131, 1342, 0, 0, 0];

        let result1 = deriver.get_extended_key(&path).unwrap();
        let result2 = deriver.get_extended_key(&path).unwrap();

        assert_eq!(result1.0, result2.0);
        assert_eq!(result1.1, result2.1);
        assert_eq!(result1.2, result2.2);
    }
}
