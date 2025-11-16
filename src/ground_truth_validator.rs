use bitcoin::bip32::{ChildNumber, DerivationPath, Xpub};
use bitcoin::secp256k1::{self, Secp256k1};
use bitcoin::{Address, CompressedPublicKey, Network, NetworkKind, PublicKey};
use crate::prefix::{AddressType, Prefix};

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

    #[cfg(test)]
    pub fn validate_address(&self, prefix: &Prefix, path: &[u32; 6]) -> Result<bool, String> {
        let derived_key = self.derive_key(path)?;

        let address = match prefix.address_type {
            AddressType::P2PKH => self.pubkey_to_p2pkh_address(&derived_key)?,
            AddressType::P2WPKH => self.pubkey_to_p2wpkh_address(&derived_key)?,
        };

        Ok(address.starts_with(prefix.as_str()))
    }

    #[cfg(test)]
    pub fn get_address(&self, path: &[u32; 6], address_type: AddressType) -> Result<String, String> {
        let derived_key = self.derive_key(path)?;
        match address_type {
            AddressType::P2PKH => self.pubkey_to_p2pkh_address(&derived_key),
            AddressType::P2WPKH => self.pubkey_to_p2wpkh_address(&derived_key),
        }
    }

    pub fn validate_and_get_address(
        &self,
        prefix: &Prefix,
        path: &[u32; 6],
    ) -> Result<Option<String>, String> {
        let derived_key = self.derive_key(path)?;

        let address = match prefix.address_type {
            AddressType::P2PKH => self.pubkey_to_p2pkh_address(&derived_key)?,
            AddressType::P2WPKH => self.pubkey_to_p2wpkh_address(&derived_key)?,
        };

        if address.starts_with(prefix.as_str()) {
            Ok(Some(address))
        } else {
            Ok(None)
        }
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

    fn pubkey_to_p2wpkh_address(
        &self,
        pubkey: &bitcoin::secp256k1::PublicKey,
    ) -> Result<String, String> {
        let compressed_pubkey = CompressedPublicKey(*pubkey);

        let address = Address::p2wpkh(&compressed_pubkey, Network::Bitcoin);

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
        let address = validator.get_address(&path, AddressType::P2PKH).unwrap();
        assert_eq!(address, "16EhLAUerc8rnmdHvBh1ABjEsddTom3FyZ");
    }

    #[test]
    fn test_address_path_1000_2000_0_0_0_1() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 1];
        let address = validator.get_address(&path, AddressType::P2PKH).unwrap();
        assert_eq!(address, "12x9m2JaDDWZ2Pf7t97hVjymA1uHRqEd7C");
    }

    #[test]
    fn test_address_path_1000_2000_0_0_0_100() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 100];
        let address = validator.get_address(&path, AddressType::P2PKH).unwrap();
        assert_eq!(address, "1K1g6s5LHneq2Km9rs8fGfGc2xpfsVRQ82");
    }

    #[test]
    fn test_address_path_1000_2000_0_1_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 1, 0, 0];
        let address = validator.get_address(&path, AddressType::P2PKH).unwrap();
        assert_eq!(address, "14hMRf1rnTgwwdEcPYUJMq5PYWh2owCo4x");
    }

    #[test]
    fn test_address_path_1000_2000_1_0_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 1, 0, 0, 0];
        let address = validator.get_address(&path, AddressType::P2PKH).unwrap();
        assert_eq!(address, "1J57PqrPQSKP85Gd8eYRSwHS65FtorCZwB");
    }

    #[test]
    fn test_address_path_1000_2000_0_0_0_9999() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 9999];
        let address = validator.get_address(&path, AddressType::P2PKH).unwrap();
        assert_eq!(address, "1ND9xQjQWC7U2xmhapTWFSEsfsDozqkp4z");
    }

    #[test]
    fn test_address_path_1000_2000_0_100_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 100, 0, 0];
        let address = validator.get_address(&path, AddressType::P2PKH).unwrap();
        assert_eq!(address, "17zbeS1wPdtncwSZCtZRptPz9MRY7ZGt9H");
    }

    #[test]
    fn test_address_path_1000_2000_0_1000_0_50() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 1000, 0, 50];
        let address = validator.get_address(&path, AddressType::P2PKH).unwrap();
        assert_eq!(address, "17DojH5JeQtfFbyG4yuiCmuwQrhdR8UfN3");
    }

    #[test]
    fn test_address_path_5000_6000_0_0_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [5000, 6000, 0, 0, 0, 0];
        let address = validator.get_address(&path, AddressType::P2PKH).unwrap();
        assert_eq!(address, "1LAVfqDqtFfjUSQUhZsE7TxWrQpgHRsGVF");
    }

    #[test]
    fn test_address_path_9999_9999_0_0_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [9999, 9999, 0, 0, 0, 0];
        let address = validator.get_address(&path, AddressType::P2PKH).unwrap();
        assert_eq!(address, "12Wq6aUM2jiJQWV3gSCGogWuAyYZR2otoH");
    }

    #[test]
    fn test_validate_address_prefix_matching() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();

        // Test path [1000, 2000, 0, 0, 0, 0] -> "16EhLAUerc8rnmdHvBh1ABjEsddTom3FyZ"
        let path = [1000, 2000, 0, 0, 0, 0];

        // Should match "1", "16", "16E", etc.
        let prefix1 = Prefix::new("1").unwrap();
        let prefix16 = Prefix::new("16").unwrap();
        let prefix16e = Prefix::new("16E").unwrap();
        assert!(validator.validate_address(&prefix1, &path).unwrap());
        assert!(validator.validate_address(&prefix16, &path).unwrap());
        assert!(validator.validate_address(&prefix16e, &path).unwrap());

        // Should NOT match other prefixes
        let prefix17 = Prefix::new("17").unwrap();
        let prefix12 = Prefix::new("12").unwrap();
        assert!(!validator.validate_address(&prefix17, &path).unwrap());
        assert!(!validator.validate_address(&prefix12, &path).unwrap());
    }

    // P2WPKH address generation tests
    #[test]
    fn test_p2wpkh_address_path_1000_2000_0_0_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 0];
        let address = validator.get_address(&path, AddressType::P2WPKH).unwrap();
        assert_eq!(address, "bc1q89hm8k39a388dju9fysuk6dsm6eerzj860sxx4");
    }

    #[test]
    fn test_p2wpkh_address_path_1000_2000_0_0_0_1() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 1];
        let address = validator.get_address(&path, AddressType::P2WPKH).unwrap();
        assert_eq!(address, "bc1qz4n9pwdcwzc9kczc7e6vk3cpxr2mfn8r9m6jzu");
    }

    #[test]
    fn test_p2wpkh_address_path_1000_2000_0_0_0_100() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 100];
        let address = validator.get_address(&path, AddressType::P2WPKH).unwrap();
        assert_eq!(address, "bc1qckfwwupesemaulllr9jrmllhxyymku2tajj6g7");
    }

    #[test]
    fn test_p2wpkh_address_path_1000_2000_0_1_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 1, 0, 0];
        let address = validator.get_address(&path, AddressType::P2WPKH).unwrap();
        assert_eq!(address, "bc1q9z9ppkny85mndyc968hcudsmgjzqlaern0mq2d");
    }

    #[test]
    fn test_p2wpkh_address_path_1000_2000_1_0_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 1, 0, 0, 0];
        let address = validator.get_address(&path, AddressType::P2WPKH).unwrap();
        assert_eq!(address, "bc1qhdqj8dyvwe3t2lt2qha073cgkyuseretjv6sjx");
    }

    #[test]
    fn test_p2wpkh_address_path_1000_2000_0_0_0_9999() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 9999];
        let address = validator.get_address(&path, AddressType::P2WPKH).unwrap();
        assert_eq!(address, "bc1qazn3hj8rscl7lxteumr65vjwnezr670j42l93a");
    }

    #[test]
    fn test_p2wpkh_address_path_1000_2000_0_100_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 100, 0, 0];
        let address = validator.get_address(&path, AddressType::P2WPKH).unwrap();
        assert_eq!(address, "bc1qfj6k9y9utp96yh4wt9ggqnhs7wxhukc4pr3304");
    }

    #[test]
    fn test_p2wpkh_address_path_1000_2000_0_1000_0_50() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 1000, 0, 50];
        let address = validator.get_address(&path, AddressType::P2WPKH).unwrap();
        assert_eq!(address, "bc1qgs7vp65du9cpeu5p242kuntaakuqsptghdzv02");
    }

    #[test]
    fn test_p2wpkh_address_path_5000_6000_0_0_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [5000, 6000, 0, 0, 0, 0];
        let address = validator.get_address(&path, AddressType::P2WPKH).unwrap();
        assert_eq!(address, "bc1q6gmppn3l84jlfetqvpe8crnf9jzdt5tmekrspf");
    }

    #[test]
    fn test_p2wpkh_address_path_9999_9999_0_0_0_0() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [9999, 9999, 0, 0, 0, 0];
        let address = validator.get_address(&path, AddressType::P2WPKH).unwrap();
        assert_eq!(address, "bc1qzzw9vkhkn5p3da5vpfat7c0e70wvxvrx79ehqw");
    }

    // Test validate_and_get_address with P2WPKH
    #[test]
    fn test_validate_and_get_address_p2wpkh_matching() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 0];
        // bc1q89hm8k39a388dju9fysuk6dsm6eerzj860sxx4

        let prefix_bc1q = Prefix::new("bc1q").unwrap();
        let prefix_bc1q89 = Prefix::new("bc1q89").unwrap();

        // Should match
        let result = validator.validate_and_get_address(&prefix_bc1q, &path).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "bc1q89hm8k39a388dju9fysuk6dsm6eerzj860sxx4");

        let result2 = validator.validate_and_get_address(&prefix_bc1q89, &path).unwrap();
        assert!(result2.is_some());
        assert_eq!(result2.unwrap(), "bc1q89hm8k39a388dju9fysuk6dsm6eerzj860sxx4");

        // Should NOT match
        let prefix_bc1qzz = Prefix::new("bc1qzz").unwrap();
        let result3 = validator.validate_and_get_address(&prefix_bc1qzz, &path).unwrap();
        assert!(result3.is_none());
    }

    // Test that P2PKH and P2WPKH addresses are different for same path
    #[test]
    fn test_p2pkh_vs_p2wpkh_different_addresses() {
        let validator = GroundTruthValidator::new(TEST_XPUB).unwrap();
        let path = [1000, 2000, 0, 0, 0, 0];

        let p2pkh = validator.get_address(&path, AddressType::P2PKH).unwrap();
        let p2wpkh = validator.get_address(&path, AddressType::P2WPKH).unwrap();

        // Should be different
        assert_ne!(p2pkh, p2wpkh);
        // P2PKH starts with 1
        assert!(p2pkh.starts_with("1"));
        // P2WPKH starts with bc1
        assert!(p2wpkh.starts_with("bc1"));
    }
}
