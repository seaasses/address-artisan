use crate::bitcoin_address_helper::BitcoinAddressHelper;
use crate::prefix_validator::PrefixValidator;
use crate::xpub::XpubWrapper;
use crate::stats_logger::StatsLogger;

pub struct VanityAddressFinder {
    prefix_validator: PrefixValidator,
    bitcoin_address_helper: BitcoinAddressHelper,
    xpub: XpubWrapper,
    initial_path: Vec<u32>,
    max_depth: u32,
    stats_logger: StatsLogger,
}

impl VanityAddressFinder {
    pub fn new(prefix: String, xpub: String, initial_path: Vec<u32>, max_depth: u32) -> Self {
        VanityAddressFinder {
            prefix_validator: PrefixValidator::new(prefix),
            bitcoin_address_helper: BitcoinAddressHelper::new(),
            xpub: XpubWrapper::new(&xpub),
            initial_path,
            max_depth,
            stats_logger: StatsLogger::new(),
        }
    }

    pub fn get_stats_logger(&self) -> &StatsLogger {
        &self.stats_logger
    }

    pub fn find_address(&mut self) -> Result<(String, String), String> {
        let mut current_path = self.initial_path.clone();
        current_path.push(0);
        current_path.push(0);
        current_path.push(0);
        loop {
            let pubkey_hash = self.xpub.get_pubkey_hash_160(current_path.clone())?;
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
                return Ok((address, path));
            }
            self.increment_path(&mut current_path);
        }
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
