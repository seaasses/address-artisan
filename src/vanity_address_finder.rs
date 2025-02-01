use crate::bitcoin_address_helper::BitcoinAddressHelper;
use crate::prefix_validator::PrefixValidator;
use crate::stats_logger::StatsLogger;
use crate::xpub::ExtendedPublicKeyDeriver;
use std::sync::Arc;

pub struct VanityAddressFinder {
    prefix_validator: PrefixValidator,
    bitcoin_address_helper: BitcoinAddressHelper,
    xpub: ExtendedPublicKeyDeriver,
    stats_logger: Arc<StatsLogger>,
    max_depth: u32,
    start_path: Vec<u32>,
}

impl VanityAddressFinder {
    pub fn new(
        prefix: String,
        xpub: String,
        max_depth: u32,
        stats_logger: Arc<StatsLogger>,
        start_path: Vec<u32>,
    ) -> Self {
        let mut start_path_extended = start_path;
        start_path_extended.push(0);
        start_path_extended.push(0);

        VanityAddressFinder {
            prefix_validator: PrefixValidator::new(prefix),
            bitcoin_address_helper: BitcoinAddressHelper::new(),
            xpub: ExtendedPublicKeyDeriver::new(&xpub),
            stats_logger,
            max_depth,
            start_path: start_path_extended,
        }
    }

    pub fn find_address(&mut self) -> Option<(String, String)> {
        let mut current_path = self.start_path.clone();

        while !self.stats_logger.should_stop() {
            if let Ok(pubkey_hash) = self.xpub.get_pubkey_hash_160(current_path.as_slice()) {
                self.stats_logger.increment_generated();

                if self.prefix_validator.is_valid(pubkey_hash) {
                    self.stats_logger.increment_found();
                    let address = self
                        .bitcoin_address_helper
                        .get_address_from_pubkey_hash(pubkey_hash);
                    let path = current_path
                        .iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<String>>()
                        .join("/");

                    return Some((address, path));
                }
                self.increment_path(&mut current_path);
            }
        }
        None
    }

    fn increment_path(&self, current_path: &mut Vec<u32>) {
        let mut last_index = current_path.len() - 1;
        if current_path[last_index] < self.max_depth {
            current_path[last_index] += 1;
        } else {
            current_path.pop();
            current_path.pop();
            last_index -= 2;
            if current_path[last_index] < self.xpub.non_hardening_max_index {
                current_path[last_index] += 1;
            } else {
                current_path.push(0);
            }
            current_path.push(0);
            current_path.push(0);
        }
    }
}
