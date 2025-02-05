use sha2::{Digest, Sha256};

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
        let address_bytes = bs58::decode(address).into_vec().unwrap();
        if address_bytes.len() != 25 {
            return None;
        }
        let pubkey_hash = address_bytes
            .iter()
            .skip(1)
            .take(20)
            .cloned()
            .collect::<Vec<u8>>();
        Some(pubkey_hash.try_into().unwrap())
    }

    fn base58_encode(&self, data: &[u8]) -> String {
        bs58::encode(data).into_string()
    }
}
