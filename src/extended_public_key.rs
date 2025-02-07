use secp256k1::PublicKey;
use sha2::{Digest, Sha256};

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

        Ok(ExtendedPubKey {
            public_key,
            chain_code,
            depth: payload[4],
        })
    }
}
