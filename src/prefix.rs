use crate::bitcoin_address_helper::{AddressEncoder, BitcoinAddressHelper};
use std::collections::HashSet;

pub trait PrefixValidator {
    fn validate_and_get_address(
        &mut self,
        prefix: &Prefix,
        pubkey_hash: &[u8; 20],
    ) -> Option<String>;
}

#[derive(Clone, Debug)]
pub struct Prefix {
    pub prefix_str: String,
    pub pattern: Vec<u8>,
}

pub struct CpuPrefixValidator {
    address_encoder: BitcoinAddressHelper,
}

impl Prefix {
    pub fn new(prefix_str: &str) -> Self {
        let helper = BitcoinAddressHelper::new();
        let pattern = Self::compute_pattern(prefix_str, &helper);

        Self {
            prefix_str: prefix_str.to_string(),
            pattern,
        }
    }

    pub fn matches_pattern(&self, pubkey_hash: &[u8; 20]) -> bool {
        if self.pattern.len() == 1 {
            return true;
        }

        for i in 0..self.pattern.len() {
            if pubkey_hash[i] != self.pattern[i] {
                return false;
            }
        }

        true
    }

    fn compute_pattern(prefix: &str, bitcoin_address_helper: &BitcoinAddressHelper) -> Vec<u8> {
        if prefix.len() == 1 {
            return vec![0x00];
        }

        let prefix_len = prefix.len();
        let mut pubkey_hashs: Vec<[u8; 20]> = Vec::new();

        for address_len in 26..=35 {
            for ones in 0..=address_len - prefix_len {
                let address = prefix.to_owned()
                    + &"1".repeat(ones)
                    + &"z".repeat(address_len - prefix_len - ones);
                if let Some(pubkey_hash) =
                    bitcoin_address_helper.get_pubkey_hash_from_address(address)
                {
                    pubkey_hashs.push(pubkey_hash);
                }
            }
        }

        for address_len in 26..=35 {
            for zs in 0..=address_len - prefix_len {
                let address = prefix.to_owned()
                    + &"z".repeat(zs)
                    + &"1".repeat(address_len - prefix_len - zs);
                if let Some(pubkey_hash) =
                    bitcoin_address_helper.get_pubkey_hash_from_address(address)
                {
                    pubkey_hashs.push(pubkey_hash);
                }
            }
        }

        let extended_prefix_length: usize = 3;
        let extended_prefix_combinations: Vec<String> =
            Self::get_all_base58_combinations(extended_prefix_length);

        for address_len in 26..=35 {
            for extended_prefix in extended_prefix_combinations.iter() {
                let address = prefix.to_owned()
                    + extended_prefix
                    + &"1".repeat(address_len - prefix_len - extended_prefix_length);
                if let Some(pubkey_hash) =
                    bitcoin_address_helper.get_pubkey_hash_from_address(address)
                {
                    pubkey_hashs.push(pubkey_hash);
                }
            }
        }

        for address_len in 26..=35 {
            for extended_prefix in extended_prefix_combinations.iter() {
                let address = prefix.to_owned()
                    + extended_prefix
                    + &"z".repeat(address_len - prefix_len - extended_prefix_length);
                if let Some(pubkey_hash) =
                    bitcoin_address_helper.get_pubkey_hash_from_address(address)
                {
                    pubkey_hashs.push(pubkey_hash);
                }
            }
        }

        if pubkey_hashs.len() < 2 {
            return vec![];
        }

        let mut final_pattern: Vec<u8> = Vec::new();
        for i in 0..=20 {
            let mut pattern_for_this_index: HashSet<Vec<u8>> = HashSet::new();
            for pubkey_hash in pubkey_hashs.iter() {
                let first_n_bytes = pubkey_hash.iter().take(i).cloned().collect::<Vec<u8>>();
                pattern_for_this_index.insert(first_n_bytes);
            }
            if pattern_for_this_index.len() == 1 {
                final_pattern = pattern_for_this_index.iter().next().unwrap().clone();
            } else {
                break;
            }
        }
        final_pattern
    }

    fn get_all_base58_combinations(n: usize) -> Vec<String> {
        let base58_chars = [
            "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F", "G", "H",
            "J", "K", "L", "M", "N", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "a",
            "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "m", "n", "o", "p", "q", "r", "s",
            "t", "u", "v", "w", "x", "y", "z",
        ];

        fn generate_combinations(
            chars: &[&str],
            current: String,
            length: usize,
            result: &mut Vec<String>,
        ) {
            if length == 0 {
                result.push(current);
                return;
            }

            for &c in chars {
                let mut new_str = current.clone();
                new_str.push_str(c);
                generate_combinations(chars, new_str, length - 1, result);
            }
        }

        let mut result = Vec::new();
        generate_combinations(&base58_chars, String::new(), n, &mut result);
        result
    }
}

impl CpuPrefixValidator {
    pub fn new() -> Self {
        Self {
            address_encoder: BitcoinAddressHelper::new(),
        }
    }
}

impl PrefixValidator for CpuPrefixValidator {
    fn validate_and_get_address(
        &mut self,
        prefix: &Prefix,
        pubkey_hash: &[u8; 20],
    ) -> Option<String> {
        if !prefix.matches_pattern(pubkey_hash) {
            return None;
        }

        let address_with_fake_checksum = self
            .address_encoder
            .get_address_with_fake_checksum(pubkey_hash);

        if !address_with_fake_checksum.starts_with(&prefix.prefix_str) {
            return None;
        }

        let real_address = self
            .address_encoder
            .get_address_from_pubkey_hash(pubkey_hash);

        Some(real_address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_creation() {
        let prefix = Prefix::new("1Test");
        assert_eq!(prefix.prefix_str, "1Test");
        assert!(!prefix.pattern.is_empty());
    }

    #[test]
    fn test_single_char_prefix() {
        let prefix = Prefix::new("1");
        assert_eq!(prefix.pattern, vec![0x00]);

        let dummy_hash = [0u8; 20];
        assert!(prefix.matches_pattern(&dummy_hash));
    }

    #[test]
    fn test_prefix_clone() {
        let prefix = Prefix::new("1ABC");
        let cloned = prefix.clone();

        assert_eq!(prefix.prefix_str, cloned.prefix_str);
        assert_eq!(prefix.pattern, cloned.pattern);
    }

    #[test]
    fn test_pattern_matching() {
        let prefix = Prefix::new("1Test");
        assert!(prefix.pattern.len() > 1);

        let non_matching_hash = [0xFFu8; 20];
        assert!(!prefix.matches_pattern(&non_matching_hash));
    }

    #[test]
    fn test_cpu_prefix_validator() {
        let prefix = Prefix::new("1");
        let mut validator = CpuPrefixValidator::new();

        let pubkey_hash = [
            0x62, 0x31, 0x50, 0x63, 0x75, 0xbc, 0x46, 0x62, 0x98, 0x2d, 0x3e, 0x08, 0x5e, 0x67,
            0x5f, 0x55, 0x7c, 0xc1, 0x99, 0xf4,
        ];

        let result = validator.validate_and_get_address(&prefix, &pubkey_hash);
        assert!(result.is_some());
        assert!(result.unwrap().starts_with("1"));
    }

    #[test]
    fn test_validator_rejects_non_matching() {
        let prefix = Prefix::new("1ZZZZ");
        let mut validator = CpuPrefixValidator::new();

        let pubkey_hash = [
            0x62, 0x31, 0x50, 0x63, 0x75, 0xbc, 0x46, 0x62, 0x98, 0x2d, 0x3e, 0x08, 0x5e, 0x67,
            0x5f, 0x55, 0x7c, 0xc1, 0x99, 0xf4,
        ];

        let result = validator.validate_and_get_address(&prefix, &pubkey_hash);
        assert!(result.is_none());
    }
}
