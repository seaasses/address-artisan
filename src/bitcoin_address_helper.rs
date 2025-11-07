use sha2::{Digest, Sha256};

pub trait AddressEncoder {
    fn get_address_from_pubkey_hash(&mut self, pubkey_hash: &[u8; 20]) -> String;
    fn get_address_with_fake_checksum(&self, pubkey_hash: &[u8; 20]) -> String;
    fn get_pubkey_hash_from_address(&self, address: String) -> Option<[u8; 20]>;
}

#[derive(Debug, Clone)]
pub struct BitcoinAddressHelper {
    sha256_hasher: Sha256,
}

impl BitcoinAddressHelper {
    pub fn new() -> Self {
        BitcoinAddressHelper {
            sha256_hasher: Sha256::new(),
        }
    }

    fn base58_encode(&self, data: &[u8]) -> String {
        bs58::encode(data).into_string()
    }
}

impl AddressEncoder for BitcoinAddressHelper {
    fn get_address_from_pubkey_hash(&mut self, pubkey_hash: &[u8; 20]) -> String {
        let mut result = [0u8; 25];
        result[0] = 0x00;
        result[1..21].copy_from_slice(pubkey_hash);

        self.sha256_hasher.reset();
        self.sha256_hasher.update(&result[0..21]);
        let first_hash = self.sha256_hasher.finalize_reset();

        self.sha256_hasher.reset();
        self.sha256_hasher.update(&first_hash);
        let second_hash = self.sha256_hasher.finalize_reset();

        result[21..25].copy_from_slice(&second_hash[0..4]);
        self.base58_encode(&result)
    }

    fn get_address_with_fake_checksum(&self, pubkey_hash: &[u8; 20]) -> String {
        let mut result = [0u8; 25];
        result[0] = 0x00;
        result[1..21].copy_from_slice(pubkey_hash);
        self.base58_encode(&result)
    }

    fn get_pubkey_hash_from_address(&self, address: String) -> Option<[u8; 20]> {
        let address_bytes = bs58::decode(address).into_vec().ok()?;

        if address_bytes.len() != 25 {
            return None;
        }

        address_bytes[1..21].try_into().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_address_from_pubkey_hash() {
        let mut helper = BitcoinAddressHelper::new();
        let pubkey_hash = [
            0x62, 0x31, 0x50, 0x63, 0x75, 0xbc, 0x46, 0x62, 0x98, 0x2d, 0x3e, 0x08, 0x5e, 0x67,
            0x5f, 0x55, 0x7c, 0xc1, 0x99, 0xf4,
        ];

        let address = helper.get_address_from_pubkey_hash(&pubkey_hash);
        assert_eq!(address, "19xCJCT3D55J6JcpAmbtDrSjiT1D2pQiPj");
    }

    #[test]
    fn test_get_address_with_fake_checksum() {
        let helper = BitcoinAddressHelper::new();
        let pubkey_hash = [
            0x62, 0x31, 0x50, 0x63, 0x75, 0xbc, 0x46, 0x62, 0x98, 0x2d, 0x3e, 0x08, 0x5e, 0x67,
            0x5f, 0x55, 0x7c, 0xc1, 0x99, 0xf4,
        ];
        let address = helper.get_address_with_fake_checksum(&pubkey_hash);

        assert_eq!(address, "19xCJCT3D55J6JcpAmbtDrSjiT1CzUPtdm");
    }

    #[test]
    fn test_get_pubkey_hash_from_address() {
        let helper = BitcoinAddressHelper::new();
        let address = "19xCJCT3D55J6JcpAmbtDrSjiT1D2pQiPj".to_string();
        let pubkey_hash = helper.get_pubkey_hash_from_address(address).unwrap();
        println!("Pubkey hash: {:?}", pubkey_hash);
        assert_eq!(
            pubkey_hash,
            [
                0x62, 0x31, 0x50, 0x63, 0x75, 0xbc, 0x46, 0x62, 0x98, 0x2d, 0x3e, 0x08, 0x5e, 0x67,
                0x5f, 0x55, 0x7c, 0xc1, 0x99, 0xf4,
            ]
        );
    }

    #[test]
    fn test_get_pubkey_hash_from_invalid_address() {
        let helper = BitcoinAddressHelper::new();
        let invalid_address = "invalid_address".to_string();
        assert!(helper
            .get_pubkey_hash_from_address(invalid_address)
            .is_none());
    }

    #[test]
    fn test_roundtrip() {
        let mut helper = BitcoinAddressHelper::new();
        let original_pubkey_hash = [
            0x62, 0x31, 0x50, 0x63, 0x75, 0xbc, 0x46, 0x62, 0x98, 0x2d, 0x3e, 0x08, 0x5e, 0x67,
            0x5f, 0x55, 0x7c, 0xc1, 0x99, 0xf4,
        ];
        let address = helper.get_address_from_pubkey_hash(&original_pubkey_hash);
        let recovered_pubkey_hash = helper.get_pubkey_hash_from_address(address).unwrap();
        assert_eq!(original_pubkey_hash, recovered_pubkey_hash);
    }
}
