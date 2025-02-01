use bitcoin::hashes::{sha256, Hash};

pub struct BitcoinAddressHelper {}

impl BitcoinAddressHelper {
    pub fn new() -> Self {
        BitcoinAddressHelper {}
    }

    pub fn get_address_from_pubkey_hash(&self, pubkey_hash: [u8; 20]) -> String {
        let mut result = Vec::with_capacity(25);
        result.push(0x00); // network byte
        result.extend_from_slice(&pubkey_hash);
        let first_hash = sha256::Hash::hash(&result);
        let second_hash = sha256::Hash::hash(first_hash.as_ref());
        let checksum = &second_hash[0..4]; // 4 bytes of a double sha256 hash
        result.extend_from_slice(checksum);
        self.base58_encode(&result)
    }

    fn base58_encode(&self, data: &[u8]) -> String {
        bs58::encode(data).into_string()
    }
}
