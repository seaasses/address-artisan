use bitcoin::bip32::{ChildNumber, DerivationPath, Xpub};
use bitcoin::secp256k1::{self, Secp256k1};
use bitcoin::{Address, NetworkKind, PublicKey};

pub struct GroundTruthValidator {
    xpub: Xpub,
    secp: Secp256k1<secp256k1::All>,
}

//Bitcoin specialized library for validating addresses derived from xpubs with my own implementation
impl GroundTruthValidator {
    pub fn new(xpub_str: &str) -> Result<Self, String> {
        let xpub = xpub_str
            .parse::<Xpub>()
            .map_err(|e| format!("Failed to parse xpub: {}", e))?;

        Ok(Self {
            xpub,
            secp: Secp256k1::new(),
        })
    }

    pub fn validate_address(&self, prefix: &str, path: &[u32; 6]) -> Result<bool, String> {
        let derived_key = self.derive_key(path)?;

        let address = self.pubkey_to_p2pkh_address(&derived_key)?;

        Ok(address.starts_with(prefix))
    }

    pub fn get_address(&self, path: &[u32; 6]) -> Result<String, String> {
        let derived_key = self.derive_key(path)?;
        self.pubkey_to_p2pkh_address(&derived_key)
    }

    fn derive_key(&self, path: &[u32; 6]) -> Result<bitcoin::secp256k1::PublicKey, String> {
        let child_numbers: Vec<ChildNumber> = path
            .iter()
            .map(|&index| ChildNumber::from_normal_idx(index).unwrap())
            .collect();

        let derivation_path = DerivationPath::from(child_numbers);

        let derived_xpub = self
            .xpub
            .derive_pub(&self.secp, &derivation_path)
            .map_err(|e| format!("Failed to derive key: {}", e))?;

        Ok(derived_xpub.public_key)
    }

    fn pubkey_to_p2pkh_address(
        &self,
        pubkey: &bitcoin::secp256k1::PublicKey,
    ) -> Result<String, String> {
        let public_key = PublicKey::new(*pubkey);

        let address = Address::p2pkh(public_key, NetworkKind::Main);

        Ok(address.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_XPUB: &str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";

    #[test]
    fn test_ground_truth_validator_creation() {
        let validator = GroundTruthValidator::new(TEST_XPUB);
        assert!(validator.is_ok());
    }

    #[test]
    fn test_ground_truth_validator_invalid_xpub() {
        let invalid_xpub = "invalid";
        let validator = GroundTruthValidator::new(invalid_xpub);
        assert!(validator.is_err());
    }

    #[test]
    fn test_address_path_1000_2000_0_0_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 0];
        let address = validator.get_address(&path).unwrap();
        assert_eq!(address, "16EhLAUerc8rnmdHvBh1ABjEsddTom3FyZ");
    }

    #[test]
    fn test_address_path_1000_2000_0_0_0_1() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 1];
        let address = validator.get_address(&path).unwrap();
        assert_eq!(address, "12x9m2JaDDWZ2Pf7t97hVjymA1uHRqEd7C");
    }

    #[test]
    fn test_address_path_1000_2000_0_0_0_100() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 100];
        let address = validator.get_address(&path).unwrap();
        assert_eq!(address, "1K1g6s5LHneq2Km9rs8fGfGc2xpfsVRQ82");
    }

    #[test]
    fn test_address_path_1000_2000_0_1_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 1, 0, 0];
        let address = validator.get_address(&path).unwrap();
        assert_eq!(address, "14hMRf1rnTgwwdEcPYUJMq5PYWh2owCo4x");
    }

    #[test]
    fn test_address_path_1000_2000_1_0_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 1, 0, 0, 0];
        let address = validator.get_address(&path).unwrap();
        assert_eq!(address, "1J57PqrPQSKP85Gd8eYRSwHS65FtorCZwB");
    }

    #[test]
    fn test_address_path_1000_2000_0_0_0_9999() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 9999];
        let address = validator.get_address(&path).unwrap();
        assert_eq!(address, "1ND9xQjQWC7U2xmhapTWFSEsfsDozqkp4z");
    }

    #[test]
    fn test_address_path_1000_2000_0_100_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 100, 0, 0];
        let address = validator.get_address(&path).unwrap();
        assert_eq!(address, "17zbeS1wPdtncwSZCtZRptPz9MRY7ZGt9H");
    }

    #[test]
    fn test_address_path_1000_2000_0_1000_0_50() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 1000, 0, 50];
        let address = validator.get_address(&path).unwrap();
        assert_eq!(address, "17DojH5JeQtfFbyG4yuiCmuwQrhdR8UfN3");
    }

    #[test]
    fn test_address_path_5000_6000_0_0_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [5000, 6000, 0, 0, 0, 0];
        let address = validator.get_address(&path).unwrap();
        assert_eq!(address, "1LAVfqDqtFfjUSQUhZsE7TxWrQpgHRsGVF");
    }

    #[test]
    fn test_address_path_9999_9999_0_0_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [9999, 9999, 0, 0, 0, 0];
        let address = validator.get_address(&path).unwrap();
        assert_eq!(address, "12Wq6aUM2jiJQWV3gSCGogWuAyYZR2otoH");
    }

    #[test]
    fn test_validate_address_prefix_matching() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();

        // Test path [1000, 2000, 0, 0, 0, 0] -> "16EhLAUerc8rnmdHvBh1ABjEsddTom3FyZ"
        let path = [1000, 2000, 0, 0, 0, 0];

        // Should match "1", "16", "16E", etc.
        assert!(validator.validate_address("1", &path).unwrap());
        assert!(validator.validate_address("16", &path).unwrap());
        assert!(validator.validate_address("16E", &path).unwrap());

        // Should NOT match other prefixes
        assert!(!validator.validate_address("17", &path).unwrap());
        assert!(!validator.validate_address("12", &path).unwrap());
    }
}
