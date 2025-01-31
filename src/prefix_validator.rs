use bs58;

pub struct PrefixValidator {
    prefix: String,
}

impl PrefixValidator {
    pub fn new(prefix: String) -> Self {
        if !prefix.starts_with("1") {
            panic!("Prefix must start with 1");
        }

        PrefixValidator { prefix: prefix }
    }

    pub fn is_valid(&self, pubkey_hash: [u8; 20]) -> bool {
        if self.prefix == "1" {
            return true; // skip prefix recognition. All addresses are valid.
        }

        let address_with_fake_checksum = Self::get_address_with_fake_checksum(pubkey_hash);
        if address_with_fake_checksum.starts_with(&self.prefix) {
            return true;
        }
        false
    }

    fn get_address_with_fake_checksum(pubkey_hash: [u8; 20]) -> String {
        let mut result = Vec::with_capacity(25);
        result.push(0x00); // version byte
        result.extend_from_slice(&pubkey_hash);
        result.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // checksum
        bs58::encode(result).into_string()
    }
}
