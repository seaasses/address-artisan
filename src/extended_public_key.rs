use secp256k1::PublicKey;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
pub struct ExtendedPubKey {
    pub public_key: PublicKey,
    pub chain_code: [u8; 32],
}

impl ExtendedPubKey {
    pub fn from_str(xpub: &str) -> Result<Self, String> {
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
        hasher.update(first_hash);
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_valid() {
        let xpub = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let extended_pub_key_result = ExtendedPubKey::from_str(xpub);
        assert!(extended_pub_key_result.is_ok());
        let extended_pub_key = extended_pub_key_result.unwrap();
        assert_eq!(
            extended_pub_key.public_key.serialize(),
            [
                0x03, 0x89, 0x44, 0xff, 0xa9, 0x54, 0x36, 0x97, 0x37, 0xa3, 0x23, 0x77, 0xf6, 0x75,
                0xb9, 0xf3, 0xe1, 0xdb, 0x85, 0xc0, 0xec, 0x5d, 0x4d, 0xdf, 0x9d, 0x92, 0xce, 0xab,
                0xa5, 0xc8, 0x23, 0xda, 0xd8
            ]
        );
        assert_eq!(
            extended_pub_key.chain_code,
            [
                0xb2, 0x97, 0x7d, 0x8e, 0x4c, 0x99, 0xcf, 0x42, 0x47, 0x92, 0xc2, 0xa2, 0x39, 0x2b,
                0x63, 0xb6, 0xd0, 0x0f, 0x32, 0x3b, 0xde, 0x39, 0x7a, 0x90, 0x3b, 0x75, 0x87, 0x84,
                0x52, 0xd4, 0x32, 0xf4
            ]
        );
    }

    #[test]
    fn test_from_str_invalid_length() {
        let xpub = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYm";
        let extended_pub_key_result = ExtendedPubKey::from_str(xpub);
        assert!(extended_pub_key_result.is_err());
    }

    #[test]
    fn test_from_str_invalid_checksum() {
        // the last character was changed to 'm'
        let xpub = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNm";
        let extended_pub_key_result = ExtendedPubKey::from_str(xpub);
        assert!(extended_pub_key_result.is_err());
    }
}
