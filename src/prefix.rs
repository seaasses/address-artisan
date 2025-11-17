use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hash160Range {
    pub low: [u8; 20],
    pub high: [u8; 20],
}

impl Hash160Range {
    pub fn new(low: [u8; 20], high: [u8; 20]) -> Self {
        Self { low, high }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AddressType {
    P2PKH,
    P2WPKH,
}

#[derive(Clone, Debug)]
pub struct Prefix {
    pub prefix_str: String,
    pub address_type: AddressType,
    pub ranges: Vec<Hash160Range>,
}

impl Prefix {
    pub fn new(prefix_str: &str) -> Result<Self, String> {
        if prefix_str.is_empty() {
            return Err("Prefix cannot be empty".to_string());
        }

        // Detect address type automatically and validate
        let address_type = if let Some(bech32_part) = prefix_str.strip_prefix("bc1q") {
            // Validate bech32 characters
            const BECH32_CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
            for c in bech32_part.chars() {
                if !BECH32_CHARSET.contains(c) {
                    return Err(format!("Invalid bech32 character: '{}'", c));
                }
            }
            AddressType::P2WPKH
        } else if prefix_str.starts_with("1") {
            // Validate base58 characters
            const VALID_BASE58_CHARS: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
            for c in prefix_str.chars() {
                if !VALID_BASE58_CHARS.contains(c) {
                    return Err(format!("Invalid base58 character: '{}'", c));
                }
            }
            AddressType::P2PKH
        } else {
            return Err("Prefix must start with '1' (P2PKH) or 'bc1q' (P2WPKH)".to_string());
        };

        // Calculate ranges based on address type
        let ranges = match address_type {
            AddressType::P2PKH => Self::get_p2pkh_ranges(prefix_str),
            AddressType::P2WPKH => Self::get_p2wpkh_ranges(prefix_str),
        };

        Ok(Self {
            prefix_str: prefix_str.to_string(),
            address_type,
            ranges,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.prefix_str
    }

    pub fn matches_pattern(&self, pubkey_hash: &[u8; 20]) -> bool {
        self.ranges
            .iter()
            .any(|range| pubkey_hash >= &range.low && pubkey_hash <= &range.high)
    }

    fn get_p2pkh_ranges(prefix: &str) -> Vec<Hash160Range> {
        let mut non_ones = prefix;
        let mut ones_count = 0;
        while non_ones.starts_with('1') {
            non_ones = &non_ones[1..];
            ones_count += 1;
        }

        let b58top = if !non_ones.is_empty() {
            let first_char = &non_ones[0..1];
            Self::base58_to_biguint(first_char).to_usize().unwrap_or(0)
        } else {
            0
        };

        let n = if !non_ones.is_empty() {
            Self::base58_to_biguint(non_ones)
        } else {
            BigUint::zero()
        };

        let ceiling_shift = 200u32 - (ones_count as u32 * 8);
        let ceiling = (BigUint::one() << ceiling_shift) - BigUint::one();

        let floor = if non_ones.is_empty() {
            BigUint::zero()
        } else {
            let floor_shift = 192u32 - (ones_count as u32 * 8);
            BigUint::one() << floor_shift
        };

        let mut b58pow = 0;
        let mut temp = ceiling.clone();
        let fifty_eight = BigUint::from(58u32);
        while temp >= fifty_eight {
            b58pow += 1;
            temp /= &fifty_eight;
        }
        let b58ceil = temp.to_usize().unwrap_or(0);

        let k = b58pow - non_ones.len();

        let (mut low, mut high) = if n > BigUint::zero() {
            let multiplier = fifty_eight.pow(k as u32);
            let low = &n * &multiplier;
            let high = (&n + BigUint::one()) * &multiplier - BigUint::one();
            (low, high)
        } else {
            (BigUint::zero(), ceiling.clone())
        };

        let mut check_upper = false;
        let mut low2 = BigUint::zero();
        let mut high2 = BigUint::zero();

        if b58top <= b58ceil {
            check_upper = true;
            low2 = &low * &fifty_eight;
            high2 = &high * &fifty_eight + BigUint::from(57u32);
        }

        if check_upper {
            if low2 > ceiling {
                check_upper = false;
            } else if high2 > ceiling {
                high2 = ceiling.clone();
            }
        }

        if high < floor {
            if !check_upper {
                return Vec::new();
            }
            low = low2.clone();
            high = high2.clone();
            check_upper = false;
        } else if low < floor {
            low = floor.clone();
        }

        low = low.max(floor.clone());
        high = high.min(ceiling.clone());

        let mut final_ranges = vec![Hash160Range::new(
            Self::address_range_to_hash160_range(&low),
            Self::address_range_to_hash160_range(&high),
        )];

        if check_upper {
            low2 = low2.max(floor);
            high2 = high2.min(ceiling);
            final_ranges.push(Hash160Range::new(
                Self::address_range_to_hash160_range(&low2),
                Self::address_range_to_hash160_range(&high2),
            ));
        }

        final_ranges.dedup();
        final_ranges
    }

    fn base58_to_biguint(base58_str: &str) -> BigUint {
        match bs58::decode(base58_str).into_vec() {
            Ok(bytes) => BigUint::from_bytes_be(&bytes),
            Err(_) => BigUint::zero(),
        }
    }

    fn address_range_to_hash160_range(num: &BigUint) -> [u8; 20] {
        let bytes = num.to_bytes_be();
        let mut full_bytes = [0u8; 25];

        if bytes.len() <= 25 {
            let offset = 25 - bytes.len();
            full_bytes[offset..].copy_from_slice(&bytes);
        } else {
            full_bytes.copy_from_slice(&bytes[bytes.len() - 25..]);
        }

        let mut result = [0u8; 20];
        result.copy_from_slice(&full_bytes[1..21]);
        result
    }

    fn get_p2wpkh_ranges(prefix: &str) -> Vec<Hash160Range> {
        // bech32 charset for encoding/decoding
        const BECH32_CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

        // Remove 'bc1q' prefix to get the actual bech32 string
        let bech32_prefix = if let Some(stripped) = prefix.strip_prefix("bc1q") {
            stripped
        } else {
            return Vec::new(); // Invalid prefix
        };

        if bech32_prefix.is_empty() {
            return vec![Hash160Range::new([0u8; 20], [0xff; 20])];
        }

        let bech32_to_int = |s: &str| -> Option<BigUint> {
            let mut value = BigUint::zero();
            let base = BigUint::from(32u32);
            for ch in s.chars() {
                let idx = BECH32_CHARSET.find(ch)?;
                value = value * &base + BigUint::from(idx);
            }
            Some(value)
        };


        // Calculate minimum and maximum by padding with 'q' (0) and 'l' (31)
        let remaining_len = 32 - bech32_prefix.len();
        let minimum_str = format!("{}{}", bech32_prefix, "q".repeat(remaining_len));
        let maximum_str = format!("{}{}", bech32_prefix, "l".repeat(remaining_len));

        let minimum_int = bech32_to_int(&minimum_str).unwrap_or_else(BigUint::zero);
        let maximum_int = bech32_to_int(&maximum_str).unwrap_or_else(BigUint::zero);

        // Convert to hash160 ranges (20 bytes)
        let low = Self::biguint_to_20_bytes(&minimum_int);
        let high = Self::biguint_to_20_bytes(&maximum_int);

        vec![Hash160Range::new(low, high)]
    }

    fn biguint_to_20_bytes(num: &BigUint) -> [u8; 20] {
        let bytes = num.to_bytes_be();
        let mut result = [0u8; 20];

        let offset = 20 - bytes.len();
        result[offset..].copy_from_slice(&bytes);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_1() {
        let prefix = Prefix::new("1").unwrap();
        assert_eq!(prefix.ranges.len(), 1);

        let expected_low = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let expected_high = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];

        assert_eq!(prefix.ranges[0].low, expected_low);
        assert_eq!(prefix.ranges[0].high, expected_high);
    }

    #[test]
    fn test_prefix_1a_uppercase() {
        let prefix = Prefix::new("1A").unwrap();
        assert_eq!(prefix.ranges.len(), 2);

        // Range 1
        let expected_low_1 = [
            0x01, 0xb3, 0xbe, 0x60, 0x3f, 0x13, 0xac, 0xde, 0xf9, 0xf6, 0xc2, 0xa7, 0xe4, 0x66,
            0x09, 0x00, 0xf6, 0x79, 0x46, 0x2e,
        ];
        let expected_high_1 = [
            0x01, 0xe4, 0x28, 0xdc, 0xb7, 0xdc, 0xf8, 0xf7, 0xc0, 0x67, 0x82, 0xf3, 0x6f, 0x8d,
            0xd1, 0x1d, 0x83, 0xa3, 0x31, 0x88,
        ];

        assert_eq!(prefix.ranges[0].low, expected_low_1);
        assert_eq!(prefix.ranges[0].high, expected_high_1);

        // Range 2
        let expected_low_2 = [
            0x62, 0xb9, 0x21, 0xce, 0x4a, 0x75, 0x2a, 0x84, 0xa1, 0xe8, 0x1a, 0x09, 0xbf, 0x1e,
            0x0a, 0x37, 0xd7, 0x79, 0xe6, 0x89,
        ];
        let expected_high_2 = [
            0x6d, 0xb1, 0x42, 0x01, 0xa8, 0x10, 0x68, 0x21, 0x97, 0x73, 0xab, 0x27, 0x46, 0x21,
            0x60, 0xaf, 0xd2, 0xf9, 0x39, 0x09,
        ];

        assert_eq!(prefix.ranges[1].low, expected_low_2);
        assert_eq!(prefix.ranges[1].high, expected_high_2);
    }

    #[test]
    fn test_prefix_1a() {
        let prefix = Prefix::new("1a").unwrap();
        assert_eq!(prefix.ranges.len(), 1);

        let expected_low = [
            0x06, 0x3d, 0xba, 0x0b, 0x91, 0xf2, 0xcf, 0x31, 0x94, 0x88, 0xc9, 0xbc, 0xf0, 0x20,
            0xcb, 0xae, 0x32, 0x67, 0x56, 0xaa,
        ];
        let expected_high = [
            0x06, 0x6e, 0x24, 0x88, 0x0a, 0xbc, 0x1b, 0x4a, 0x5a, 0xf9, 0x8a, 0x08, 0x7b, 0x48,
            0x93, 0xca, 0xbf, 0x91, 0x42, 0x04,
        ];

        assert_eq!(prefix.ranges[0].low, expected_low);
        assert_eq!(prefix.ranges[0].high, expected_high);
    }

    #[test]
    fn test_prefix_1ab() {
        let prefix = Prefix::new("1ab").unwrap();
        assert_eq!(prefix.ranges.len(), 1);

        let expected_low = [
            0x06, 0x5a, 0x1b, 0xc7, 0x4b, 0x83, 0x4b, 0x40, 0x1a, 0x84, 0x43, 0x4a, 0x53, 0x5b,
            0x6d, 0x20, 0x09, 0x91, 0x91, 0x2f,
        ];
        let expected_high = [
            0x06, 0x5a, 0xf1, 0x79, 0xfe, 0x25, 0xa9, 0x40, 0x87, 0xde, 0x7b, 0x92, 0x3f, 0xaf,
            0xf9, 0x67, 0x26, 0x7c, 0x38, 0x8d,
        ];

        assert_eq!(prefix.ranges[0].low, expected_low);
        assert_eq!(prefix.ranges[0].high, expected_high);
    }

    #[test]
    fn test_prefix_1seaasses_uppercase() {
        let prefix = Prefix::new("1SEAASSES").unwrap();
        assert_eq!(prefix.ranges.len(), 1);

        let expected_low = [
            0x04, 0xc5, 0x61, 0xfd, 0x54, 0x91, 0x4f, 0xe0, 0xf3, 0xf5, 0x34, 0x8d, 0x09, 0x79,
            0x6a, 0x2e, 0x1d, 0x92, 0x56, 0xb4,
        ];
        let expected_high = [
            0x04, 0xc5, 0x61, 0xfd, 0x54, 0x91, 0x67, 0xfd, 0x0b, 0x8b, 0x6e, 0xdf, 0x30, 0x36,
            0xd1, 0x0a, 0x39, 0x94, 0x2a, 0x5c,
        ];

        assert_eq!(prefix.ranges[0].low, expected_low);
        assert_eq!(prefix.ranges[0].high, expected_high);
    }

    // P2WPKH tests
    #[test]
    fn test_address_type_detection_p2pkh() {
        let prefix = Prefix::new("1abc").unwrap();
        assert_eq!(prefix.address_type, AddressType::P2PKH);
    }

    #[test]
    fn test_address_type_detection_p2wpkh() {
        let prefix = Prefix::new("bc1qaaa").unwrap();
        assert_eq!(prefix.address_type, AddressType::P2WPKH);
    }

    #[test]
    fn test_p2wpkh_prefix_all_addresses() {
        let prefix = Prefix::new("bc1q").unwrap();
        assert_eq!(prefix.address_type, AddressType::P2WPKH);
        assert_eq!(prefix.ranges.len(), 1);

        let expected_low = [0x00; 20];
        let expected_high = [0xff; 20];

        assert_eq!(prefix.ranges[0].low, expected_low);
        assert_eq!(prefix.ranges[0].high, expected_high);
    }

    #[test]
    fn test_p2wpkh_prefix_aaa() {
        let prefix = Prefix::new("bc1qaaa").unwrap();
        assert_eq!(prefix.address_type, AddressType::P2WPKH);
        assert_eq!(prefix.ranges.len(), 1);

        // From Python test vectors
        let expected_low = [
            0xef, 0x7a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let expected_high = [
            0xef, 0x7b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];

        assert_eq!(prefix.ranges[0].low, expected_low);
        assert_eq!(prefix.ranges[0].high, expected_high);
    }

    #[test]
    fn test_p2wpkh_prefix_xyz() {
        let prefix = Prefix::new("bc1qxyz").unwrap();
        assert_eq!(prefix.address_type, AddressType::P2WPKH);
        assert_eq!(prefix.ranges.len(), 1);

        // From Python test vectors
        let expected_low = [
            0x31, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let expected_high = [
            0x31, 0x05, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];

        assert_eq!(prefix.ranges[0].low, expected_low);
        assert_eq!(prefix.ranges[0].high, expected_high);
    }

    // Validation tests
    #[test]
    fn test_invalid_prefix_empty() {
        let result = Prefix::new("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn test_invalid_prefix_wrong_start() {
        let result = Prefix::new("3abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must start with"));
    }

    #[test]
    fn test_invalid_p2pkh_character() {
        let result = Prefix::new("1abc0");  // '0' is not valid in base58
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid base58 character"));
    }

    #[test]
    fn test_invalid_p2wpkh_character() {
        let result = Prefix::new("bc1qabc");  // 'b' and 'c' are not valid in bech32
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid bech32 character"));
    }

    #[test]
    fn test_p2wpkh_valid_charset() {
        // All these should be valid bech32 characters
        let valid_chars = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
        for ch in valid_chars.chars() {
            let prefix_str = format!("bc1q{}", ch);
            let result = Prefix::new(&prefix_str);
            assert!(result.is_ok(), "Character '{}' should be valid in bech32", ch);
        }
    }
}
