use crate::bitcoin_address_helper::BitcoinAddressHelper;

pub struct VanityAddress {
    prefix: String,
    bitcoin_address_helper: BitcoinAddressHelper,
}

impl VanityAddress {
    pub fn new(prefix: &str) -> Self {
        VanityAddress {
            prefix: prefix.to_string(),
            bitcoin_address_helper: BitcoinAddressHelper::new(),
        }
    }

    pub fn get_vanity_address(&self, pubkey_hash: [u8; 20]) -> Option<String> {
        let address_with_fake_checksum = self
            .bitcoin_address_helper
            .get_address_with_fake_checksum(pubkey_hash);
        if address_with_fake_checksum.starts_with(&self.prefix) {
            let real_address = self
                .bitcoin_address_helper
                .get_address_from_pubkey_hash(pubkey_hash);
            return Some(real_address);
        }
        None
    }
}
