use crate::bitcoin_address_helper::BitcoinAddressHelper;

pub struct VanityAddressBuilder {
    prefix: String,
    bitcoin_address_helper: BitcoinAddressHelper,
}

impl VanityAddressBuilder {
    pub fn new(prefix: String) -> Self {
        if !prefix.starts_with("1") {
            panic!("Prefix must start with 1");
        }

        VanityAddressBuilder {
            prefix: prefix,
            bitcoin_address_helper: BitcoinAddressHelper::new(),
        }
    }

    pub fn get_from_pubkey_hash(&self, pubkey_hash: [u8; 20]) -> Option<String> {
        if self.prefix == "1" {
            return Some(
                self.bitcoin_address_helper
                    .get_address_from_pubkey_hash(pubkey_hash),
            );
        }

        let address_with_fake_checksum = self
            .bitcoin_address_helper
            .get_address_with_fake_checksum(pubkey_hash);

        if address_with_fake_checksum.starts_with(&self.prefix) {
            return Some(address_with_fake_checksum);
        }
        None
    }
}
