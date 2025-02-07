use clap::Parser;

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "A tool for generating vanity P2PKH Bitcoin addresses."
)]

pub struct Cli {
    #[arg(short = 'p', long = "prefix", help = "Prefix for the address", value_parser = Cli::validate_prefix)]
    pub prefix: String,
    #[arg(short = 'x', long = "xpub", help = "Xpub", value_parser = Cli::validate_xpub)]
    pub xpub: String,
    #[arg(
        short = 'm',
        long = "max-depth",
        help = "Max depth for the path's last index",
        default_value = "1000",
        value_parser = Cli::validate_max_depth
    )]
    pub max_depth: u32,
    #[arg(
        long = "i-am-boring",
        help = "Hmmm, so you are a boring person and don't have friends? Ok, I will not talk to you.",
        value_parser = Cli::fix_i_am_boring
    )]
    pub i_am_boring: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
    fn fix_i_am_boring(i_am_boring: &str) -> Result<bool, String> {
        if i_am_boring == "true" {
            return Ok(false);
        }
        if i_am_boring == "false" {
            return Ok(true);
        }
        Err("Invalid value for i-am-boring".to_string())
    }

    fn validate_max_depth(max_depth: &str) -> Result<u32, String> {
        let max_depth_int: u32 = match max_depth.starts_with("0x") {
            true => u32::from_str_radix(&max_depth[2..], 16)
                .map_err(|e: std::num::ParseIntError| e.to_string())?,
            false => max_depth
                .parse()
                .map_err(|e: std::num::ParseIntError| e.to_string())?,
        };

        if max_depth_int > 0x80000000 {
            return Err("Max depth must be less or equal to 2^31".to_string());
        }

        Ok(max_depth_int)
    }

    fn validate_prefix(prefix: &str) -> Result<String, String> {
        let valid_base58_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

        if prefix.is_empty() {
            return Err("Prefix cannot be empty".to_string());
        }

        for c in prefix.chars() {
            if !valid_base58_chars.contains(c) {
                return Err(format!("Invalid character: {}", c));
            }
        }

        if !prefix.starts_with("1") {
            return Err("Prefix must start with 1".to_string());
        }

        Ok(prefix.to_string())
    }

    fn validate_xpub(xpub: &str) -> Result<String, String> {
        let valid_base58_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        if xpub.is_empty() {
            return Err("Xpub cannot be empty".to_string());
        }
        if !xpub.starts_with("xpub") {
            return Err("Xpub should start with 'xpub'.".to_string());
        }

        for c in xpub.chars() {
            if !valid_base58_chars.contains(c) {
                return Err(format!("Invalid xpub"));
            }
        }

        Ok(xpub.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // prefix tests
    #[test]
    fn test_validate_prefix_starts_with_1() {
        let prefix = "123456789ABCDE";
        let result = Cli::validate_prefix(prefix);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_prefix_starts_with_1_failed() {
        let prefix = "23456789ABCDE";
        let result = Cli::validate_prefix(prefix);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_prefix_empty() {
        let prefix = "";
        let result = Cli::validate_prefix(prefix);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_prefix_invalid_character() {
        let prefix = "123456789ABCDE!";
        let result = Cli::validate_prefix(prefix);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_prefix_invalid_base58() {
        let prefix = "123456789ABClE";
        let result = Cli::validate_prefix(prefix);
        assert!(result.is_err());
    }

    // xpub tests
    #[test]
    fn test_validate_xpub_starts_with_xpub() {
        let xpub = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let result = Cli::validate_xpub(xpub);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_xpub_empty() {
        let xpub = "";
        let result = Cli::validate_xpub(xpub);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_xpub_invalid_base58() {
        let xpub = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn!";
        let result = Cli::validate_xpub(xpub);
        assert!(result.is_err());
    }

    // test max depth
    // as int
    #[test]
    fn test_validate_max_depth_valid() {
        let max_depth = "1000";
        let result = Cli::validate_max_depth(max_depth);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000);
    }

    #[test]
    fn test_validate_max_depth_valid_max() {
        let max_depth: u32 = 0x80000000; // index 0x7FFFFFFF
        let result = Cli::validate_max_depth(&max_depth.to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0x80000000);
    }

    #[test]
    fn test_validate_max_depth_invalid() {
        let max_depth: u32 = 0x80000000 + 1; // index 0x7FFFFFFF
        let result = Cli::validate_max_depth(&max_depth.to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_max_depth_invalid_string() {
        let max_depth = "abc";
        let result = Cli::validate_max_depth(max_depth);
        assert!(result.is_err());
    }
    // as hex
    #[test]
    fn test_validate_max_depth_valid_hex() {
        let max_depth = "0x32";
        let result = Cli::validate_max_depth(max_depth);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50);
    }

    #[test]
    fn test_validate_max_depth_invalid_hex() {
        let max_depth = "0x100U0";
        let result = Cli::validate_max_depth(max_depth);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_max_depth_valid_hex_max() {
        let max_depth = "0x80000000";
        let result = Cli::validate_max_depth(max_depth);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0x80000000);
    }

    #[test]
    fn test_validate_max_depth_invalid_hex_max() {
        let max_depth = "0x80000001";
        let result = Cli::validate_max_depth(max_depth);
        assert!(result.is_err());
    }
}
