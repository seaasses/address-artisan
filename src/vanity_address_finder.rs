use crate::stats_logger::StatsLogger;
use crate::vanity_address_builder::VanityAddressBuilder;
use crate::xpub::ExtendedPublicKeyDeriver;
use std::fmt::Write;
use std::sync::Arc;

pub struct VanityAddressFinder {
    vanity_address_builder: VanityAddressBuilder,
    xpub: ExtendedPublicKeyDeriver,
    stats_logger: Arc<StatsLogger>,
    max_index: u32,
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
        let mut start_path_extended = Vec::with_capacity(start_path.len() + 2);
        start_path_extended.extend_from_slice(&start_path);
        start_path_extended.push(0);
        start_path_extended.push(0);

        VanityAddressFinder {
            vanity_address_builder: VanityAddressBuilder::new(prefix),
            xpub: ExtendedPublicKeyDeriver::new(&xpub),
            stats_logger,
            max_index: max_depth - 1,
            start_path: start_path_extended,
        }
    }

    pub fn find_address(&mut self) -> Option<(String, String)> {
        let mut current_path = self.start_path.clone();
        let mut path_string = String::with_capacity(32); // Pre-allocate string buffer

        while !self.stats_logger.should_stop() {
            if let Ok(pubkey_hash) = self.xpub.get_pubkey_hash_160(current_path.as_slice()) {
                self.stats_logger.increment_generated();

                if let Some(address) =
                    self.vanity_address_builder.get_from_pubkey_hash(pubkey_hash)
                {
                    self.stats_logger.increment_found();


                    // Build path string more efficiently
                    path_string.clear();
                    for (i, num) in current_path.iter().enumerate() {
                        if i > 0 {
                            path_string.push('/');
                        }
                        write!(path_string, "{}", num).unwrap();
                    }

                    return Some((address, path_string));
                }
                self.increment_path(&mut current_path);
            }
        }
        None
    }

    fn increment_path(&self, current_path: &mut Vec<u32>) {
        let last_index = current_path.len() - 1;
        if current_path[last_index] < self.max_index {
            current_path[last_index] += 1;
            return;
        }

        current_path.truncate(current_path.len() - 2);
        let last_index = current_path.len() - 1;

        if current_path[last_index] < self.xpub.non_hardening_max_index {
            current_path[last_index] += 1;
        } else {
            current_path.push(0);
        }
        current_path.push(0);
        current_path.push(0);
    }
}
