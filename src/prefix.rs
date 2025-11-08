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

#[derive(Clone, Debug)]
pub struct Prefix {
    pub prefix_str: String,
    pub ranges: Vec<Hash160Range>,
}

impl Prefix {
    pub fn new(prefix_str: &str) -> Self {
        let ranges = Self::get_ranges(prefix_str);

        Self {
            prefix_str: prefix_str.to_string(),
            ranges,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.prefix_str
    }

    pub fn matches_pattern(&self, pubkey_hash: &[u8; 20]) -> bool {
        self.ranges
            .iter()
            .any(|range| pubkey_hash >= &range.low && pubkey_hash <= &range.high)
    }

    fn get_ranges(prefix: &str) -> Vec<Hash160Range> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_1() {
        let prefix = Prefix::new("1");
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
        let prefix = Prefix::new("1A");
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
        let prefix = Prefix::new("1a");
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
        let prefix = Prefix::new("1ab");
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
        let prefix = Prefix::new("1SEAASSES");
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
}
