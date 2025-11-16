use crate::constants::NON_HARDENED_MAX_INDEX;
use crate::extended_public_key::ExtendedPubKey;
use crate::prefix::Prefix;

#[derive(Clone)]
pub struct WorkbenchConfig {
    pub xpub: ExtendedPubKey,
    pub prefix: Prefix,
    pub seed0: u32,
    pub seed1: u32,
    pub max_depth: u32,
}

impl WorkbenchConfig {
    pub fn new(
        xpub: ExtendedPubKey,
        prefix: Prefix,
        seed0: u32,
        seed1: u32,
        max_depth: u32,
    ) -> Self {
        assert!(seed0 <= NON_HARDENED_MAX_INDEX, "seed0 must be <= 0x7FFFFFFF");
        assert!(seed1 <= NON_HARDENED_MAX_INDEX, "seed1 must be <= 0x7FFFFFFF");
        assert!(
            max_depth <= NON_HARDENED_MAX_INDEX,
            "max_depth must be <= 0x7FFFFFFF"
        );

        Self {
            xpub,
            prefix,
            seed0,
            seed1,
            max_depth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_seeds() {
        let seed0 = 1000u32;
        let seed1 = 2000u32;
        assert!(seed0 <= NON_HARDENED_MAX_INDEX);
        assert!(seed1 <= NON_HARDENED_MAX_INDEX);
    }

    #[test]
    #[should_panic(expected = "seed0 must be <= 0x7FFFFFFF")]
    fn test_invalid_seed0() {
        use crate::extended_public_key::ExtendedPubKey;
        use crate::prefix::Prefix;

        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1").unwrap();

        WorkbenchConfig::new(xpub, prefix, 0x80000000, 1000, 1000);
    }

    #[test]
    #[should_panic(expected = "seed1 must be <= 0x7FFFFFFF")]
    fn test_invalid_seed1() {
        use crate::extended_public_key::ExtendedPubKey;
        use crate::prefix::Prefix;

        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1").unwrap();

        WorkbenchConfig::new(xpub, prefix, 1000, 0x80000000, 1000);
    }

    #[test]
    #[should_panic(expected = "max_depth must be <= 0x7FFFFFFF")]
    fn test_invalid_max_depth() {
        use crate::extended_public_key::ExtendedPubKey;
        use crate::prefix::Prefix;

        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1").unwrap();

        WorkbenchConfig::new(xpub, prefix, 1000, 1000, 0x80000000);
    }

    #[test]
    fn test_max_valid_values() {
        use crate::extended_public_key::ExtendedPubKey;
        use crate::prefix::Prefix;

        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1").unwrap();

        let config = WorkbenchConfig::new(xpub, prefix, NON_HARDENED_MAX_INDEX, NON_HARDENED_MAX_INDEX, NON_HARDENED_MAX_INDEX);
        assert_eq!(config.seed0, NON_HARDENED_MAX_INDEX);
        assert_eq!(config.seed1, NON_HARDENED_MAX_INDEX);
        assert_eq!(config.max_depth, NON_HARDENED_MAX_INDEX);
    }
}
