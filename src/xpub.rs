use bitcoin::bip32::{ChildNumber, DerivationPath, Xpub};
use bitcoin::hashes::{ripemd160, sha256, Hash};
use secp256k1::Secp256k1;
use std::str::FromStr;

pub struct XpubWrapper {
    xpub: String,
}

impl XpubWrapper {
    pub fn new(xpub: &str) -> Self {
        Self {
            xpub: xpub.to_string(),
        }
    }

    pub fn get_pubkey_hash_160(&self, path: Vec<u32>) -> Result<[u8; 20], String> {
        let pubkey = self.get_pubkey(path)?;
        let pubkey_hash_sha256 = sha256::Hash::hash(&pubkey);
        let pubkey_hash_ripemd160 = ripemd160::Hash::hash(pubkey_hash_sha256.as_ref());
        Ok(pubkey_hash_ripemd160.to_byte_array())
    }

    pub fn get_pubkey(&self, path: Vec<u32>) -> Result<[u8; 33], String> {
        let child_numbers: Vec<ChildNumber> = path
            .into_iter()
            .map(|i| {
                if i >= 0x80000000 {
                    return Err(format!("{} is reserved for hardened derivation", i));
                }
                Ok(ChildNumber::from_normal_idx(i).unwrap())
            })
            .collect::<Result<Vec<_>, String>>()?;

        let path = DerivationPath::from_iter(child_numbers);
        let secp = Secp256k1::new();
        let xpub = match Xpub::from_str(&self.xpub) {
            Ok(xpub) => xpub,
            Err(e) => {
                return Err(format!("Failed to parse xpub: {}", e));
            }
        };
        let derived_xpub = xpub.derive_pub(&secp, &path).map_err(|e| e.to_string())?;
        Ok(derived_xpub.public_key.serialize())
    }
}
