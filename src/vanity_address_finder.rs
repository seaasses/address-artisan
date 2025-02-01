use crate::bitcoin_address_helper::BitcoinAddressHelper;
use crate::prefix_validator::PrefixValidator;
use crate::xpub::XpubWrapper;

pub struct VanityAddressFinder {
    prefix_validator: PrefixValidator,
    bitcoin_address_helper: BitcoinAddressHelper,
    xpub: XpubWrapper,
    initial_path: Vec<u32>,
}

impl VanityAddressFinder {
    pub fn new(prefix: String, xpub: String, initial_path: Vec<u32>) -> Self {
        VanityAddressFinder {
            prefix_validator: PrefixValidator::new(prefix),
            bitcoin_address_helper: BitcoinAddressHelper::new(),
            xpub: XpubWrapper::new(&xpub),
            initial_path,
        }
    }

    pub fn find_address(&self) -> Result<(String, String), String> {
        let mut current_path = self.initial_path.clone();
        current_path.push(0);
        loop {
            let pubkey_hash = self.xpub.get_pubkey_hash_160(current_path.clone())?;
            if self.prefix_validator.is_valid(pubkey_hash) {
                let address = self
                    .bitcoin_address_helper
                    .get_address_from_pubkey_hash(pubkey_hash);
                let path = current_path
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<String>>()
                    .join("/");
                return Ok((address, path));
            }
            let last_index = current_path.len() - 1;
            current_path[last_index] += 1;
        }
    }
}
