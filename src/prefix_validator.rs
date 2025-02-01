use bs58;
use std::collections::HashSet;

pub struct PrefixValidator {
    pattern: Vec<u8>,
    prefix: String,
}

impl PrefixValidator {
    pub fn new(prefix: String) -> Self {
        if !prefix.starts_with("1") {
            panic!("Prefix must start with 1");
        }

        PrefixValidator {
            pattern: Self::get_pattern(prefix.clone()),
            prefix: prefix,
        }
    }

    pub fn is_valid(&self, pubkey_hash: [u8; 20]) -> bool {
        if self.match_pattern(pubkey_hash) {
            let address_with_fake_checksum = Self::get_address_with_fake_checksum(pubkey_hash);
            return address_with_fake_checksum.starts_with(&self.prefix);
        }
        false
    }

    fn match_pattern(&self, pubkey_hash: [u8; 20]) -> bool {
        if self.pattern.len() == 1 {
            return true; // all addresses are valid. Skip pattern recognition
        }
        for i in 0..self.pattern.len() {
            if pubkey_hash[i] != self.pattern[i] {
                return false;
            }
        }
        true
    }

    fn get_pattern(prefix: String) -> Vec<u8> {
        if prefix.len() == 1 {
            // all addresses are valid. Skip the pattern search
            println!("to aqui");
            return vec![0x00];
        }

        let prefix_len = prefix.len();
        let mut pubkey_hashs: Vec<[u8; 20]> = Vec::new();

        // prefix + ones + zeros
        for address_len in 26..=35 {
            for ones in 0..=address_len - prefix_len {
                let address = prefix.as_str().to_owned()
                    + &"1".repeat(ones)
                    + &"z".repeat(address_len - prefix_len - ones);
                let pubkey_hash = Self::get_pubkey_hash_from_address(address);
                if pubkey_hash.is_some() {
                    pubkey_hashs.push(pubkey_hash.unwrap());
                }
            }
        }

        // prefix + zeros + ones
        for address_len in 26..=35 {
            for zs in 0..=address_len - prefix_len {
                let address = prefix.as_str().to_owned()
                    + &"z".repeat(zs)
                    + &"1".repeat(address_len - prefix_len - zs);
                let pubkey_hash = Self::get_pubkey_hash_from_address(address);
                if pubkey_hash.is_some() {
                    pubkey_hashs.push(pubkey_hash.unwrap());
                }
            }
        }

        let extended_prefix_length: usize = 3;
        let extended_prefix_combinations: Vec<String> =
            Self::get_all_base58_combinations(extended_prefix_length);

        // prefix + extended_prefix + ones
        for address_len in 26..=35 {
            for extended_prefix in extended_prefix_combinations.iter() {
                let address = prefix.as_str().to_owned()
                    + extended_prefix
                    + &"1".repeat(address_len - prefix_len - extended_prefix_length);
                let pubkey_hash = Self::get_pubkey_hash_from_address(address);
                if pubkey_hash.is_some() {
                    pubkey_hashs.push(pubkey_hash.unwrap());
                }
            }
        }

        // prefix + extended_prefix + zs
        for address_len in 26..=35 {
            for extended_prefix in extended_prefix_combinations.iter() {
                let address = prefix.as_str().to_owned()
                    + extended_prefix
                    + &"z".repeat(address_len - prefix_len - extended_prefix_length);
                let pubkey_hash = Self::get_pubkey_hash_from_address(address);
                if pubkey_hash.is_some() {
                    pubkey_hashs.push(pubkey_hash.unwrap());
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
        println!("final_pattern: {:?}", final_pattern);

        final_pattern
    }

    fn get_pubkey_hash_from_address(address: String) -> Option<[u8; 20]> {
        let address_bytes = bs58::decode(address).into_vec().unwrap();
        if address_bytes.len() != 25 {
            return None;
        }
        let pubkey_hash = address_bytes
            .iter()
            .skip(1)
            .take(20)
            .cloned()
            .collect::<Vec<u8>>();

        Some(pubkey_hash.try_into().unwrap())
    }

    fn get_address_with_fake_checksum(pubkey_hash: [u8; 20]) -> String {
        let mut result = Vec::with_capacity(25);
        result.push(0x00); // version byte
        result.extend_from_slice(&pubkey_hash);
        result.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // checksum
        bs58::encode(result).into_string()
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
