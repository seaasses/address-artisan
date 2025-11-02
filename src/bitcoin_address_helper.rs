use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
pub struct BitcoinAddressHelper {}

impl BitcoinAddressHelper {
    pub fn new() -> Self {
        BitcoinAddressHelper {}
    }

    pub fn get_address_from_pubkey_hash(&self, pubkey_hash: [u8; 20]) -> String {
        let mut result = Vec::with_capacity(25);
        result.push(0x00); // network byte
        result.extend_from_slice(&pubkey_hash);

        let mut hasher = Sha256::new();
        hasher.update(&result);
        let first_hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(&first_hash);
        let second_hash = hasher.finalize();

        result.extend_from_slice(&second_hash[0..4]); // 4 bytes of a double sha256 hash
        self.base58_encode(&result)
    }

    pub fn get_address_with_fake_checksum(&self, pubkey_hash: [u8; 20]) -> String {
        let mut result = Vec::with_capacity(25);
        result.push(0x00); // version byte
        result.extend_from_slice(&pubkey_hash);
        result.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // checksum
        self.base58_encode(&result)
    }

    pub fn get_pubkey_hash_from_address(&self, address: String) -> Option<[u8; 20]> {
        let address_bytes = bs58::decode(address).into_vec().ok()?;

        if address_bytes.len() != 25 {
            return None;
        }

        address_bytes[1..21].try_into().ok()
    }

    fn base58_encode(&self, data: &[u8]) -> String {
        bs58::encode(data).into_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_address_from_pubkey_hash() {
        let helper = BitcoinAddressHelper::new();
        let pubkey_hash = [
            0x62, 0x31, 0x50, 0x63, 0x75, 0xbc, 0x46, 0x62, 0x98, 0x2d, 0x3e, 0x08, 0x5e, 0x67,
            0x5f, 0x55, 0x7c, 0xc1, 0x99, 0xf4,
        ];

        let address = helper.get_address_from_pubkey_hash(pubkey_hash);
        assert_eq!(address, "19xCJCT3D55J6JcpAmbtDrSjiT1D2pQiPj");
    }

    #[test]
    fn test_get_address_with_fake_checksum() {
        let helper = BitcoinAddressHelper::new();
        let pubkey_hash = [
            0x62, 0x31, 0x50, 0x63, 0x75, 0xbc, 0x46, 0x62, 0x98, 0x2d, 0x3e, 0x08, 0x5e, 0x67,
            0x5f, 0x55, 0x7c, 0xc1, 0x99, 0xf4,
        ];
        let address = helper.get_address_with_fake_checksum(pubkey_hash);

        // with fake checksum is different from real one, but the init of the address is the samer
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
        let helper = BitcoinAddressHelper::new();
        let original_pubkey_hash = [
            0x62, 0x31, 0x50, 0x63, 0x75, 0xbc, 0x46, 0x62, 0x98, 0x2d, 0x3e, 0x08, 0x5e, 0x67,
            0x5f, 0x55, 0x7c, 0xc1, 0x99, 0xf4,
        ];
        let address = helper.get_address_from_pubkey_hash(original_pubkey_hash);
        let recovered_pubkey_hash = helper.get_pubkey_hash_from_address(address).unwrap();
        assert_eq!(original_pubkey_hash, recovered_pubkey_hash);
    }
}
