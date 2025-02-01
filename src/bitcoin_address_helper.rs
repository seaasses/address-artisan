use sha2::{Sha256, Digest};

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

    fn base58_encode(&self, data: &[u8]) -> String {
        bs58::encode(data).into_string()
    }
}
