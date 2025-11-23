use crate::prefix::Prefix;
use clap::Parser;

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "A tool for generating vanity Bitcoin addresses (P2PKH and P2WPKH)."
)]
pub struct Cli {
    #[arg(
        short = 'p',
        long = "prefix",
        help = "Prefix(es) for the address (P2PKH: '1abc...' or P2WPKH: 'bc1qaaa...'). Can specify multiple: --prefix 1A 1B or --prefix 1A,1B",
        num_args = 1..,
        value_delimiter = ',',
        value_parser = Cli::validate_prefix
    )]
    pub prefixes: Vec<Prefix>,
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
        short = 't',
        long = "cpu-threads",
        help = "Number of CPU threads to use (0 = auto-detect all available threads)",
        default_value = "0",
        value_parser = Cli::validate_cpu_threads
    )]
    pub cpu_threads: u32,
    #[arg(
        short = 'g',
        long = "gpu",
        help = "Enable GPU processing (excludes integrated/onboard GPUs). Can optionally specify GPU IDs: --gpu 0,1 or --gpu 0 1",
        num_args = 0..,
        value_delimiter = ',',
        value_parser = Cli::validate_gpu_id
    )]
    pub gpu: Option<Vec<usize>>,
    #[arg(
        long = "gpu-only",
        help = "Use only GPU (no CPU, excludes integrated/onboard GPUs)",
        default_value = "false"
    )]
    pub gpu_only: bool,
    #[arg(
        short = 'n',
        long = "num-addresses",
        help = "Number of addresses to find before stopping automatically (default: 1, 0 = never stop)",
        default_value = "1",
        value_parser = Cli::validate_num_addresses
    )]
    pub num_addresses: u32,
}

impl Cli {
    pub fn parse_args() -> Self {
        let cli = Self::parse();
        if let Err(msg) = cli.validate_conflicting_options() {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
        cli
    }

    fn validate_conflicting_options(&self) -> Result<(), String> {
        // Check for conflicting --gpu-only and -t/--cpu-threads
        if self.gpu_only && self.cpu_threads != 0 {
            return Err(
                "Error: --gpu-only and -t/--cpu-threads cannot be used together.\n       \
                --gpu-only means no CPU processing, so CPU threads are not applicable."
                    .to_string(),
            );
        }

        // Check prefix count limits
        if self.prefixes.is_empty() {
            return Err("Error: At least one prefix must be provided.".to_string());
        }

        if self.prefixes.len() > 256 {
            return Err(format!(
                "Error: Maximum 256 prefixes allowed, but {} were provided.",
                self.prefixes.len()
            ));
        }

        Ok(())
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

    fn validate_prefix(prefix: &str) -> Result<Prefix, String> {
        Prefix::new(prefix)
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
                return Err("Invalid xpub".to_string());
            }
        }

        Ok(xpub.to_string())
    }

    fn validate_cpu_threads(threads: &str) -> Result<u32, String> {
        let threads_int: u32 = threads
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;

        Ok(threads_int)
    }

    fn validate_gpu_id(id: &str) -> Result<usize, String> {
        let id_int: usize = id
            .parse()
            .map_err(|e: std::num::ParseIntError| format!("Invalid GPU ID '{}': {}", id, e))?;

        // GPU IDs should be reasonable (e.g., less than 100)
        if id_int >= 100 {
            return Err(format!("GPU ID {} seems unreasonably high", id_int));
        }

        Ok(id_int)
    }

    fn validate_num_addresses(num: &str) -> Result<u32, String> {
        let num_int: u32 = num
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;

        Ok(num_int)
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

    // P2WPKH prefix tests
    #[test]
    fn test_validate_prefix_p2wpkh_valid() {
        let prefix = "bc1qaaa";
        let result = Cli::validate_prefix(prefix);
        assert!(result.is_ok());
        let prefix_obj = result.unwrap();
        assert_eq!(prefix_obj.address_type, crate::prefix::AddressType::P2WPKH);
    }

    #[test]
    fn test_validate_prefix_p2wpkh_invalid_char() {
        let prefix = "bc1qabc"; // 'b' and 'c' not valid in bech32
        let result = Cli::validate_prefix(prefix);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_prefix_p2wpkh_all() {
        let prefix = "bc1q";
        let result = Cli::validate_prefix(prefix);
        assert!(result.is_ok());
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

    // cpu_threads tests
    #[test]
    fn test_validate_cpu_threads_zero_auto_detect() {
        let threads = "0";
        let result = Cli::validate_cpu_threads(threads);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0, "Should return 0 for auto-detect");
    }

    #[test]
    fn test_validate_cpu_threads_valid() {
        let threads = "4";
        let result = Cli::validate_cpu_threads(threads);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);
    }

    #[test]
    fn test_validate_cpu_threads_invalid() {
        let threads = "abc";
        let result = Cli::validate_cpu_threads(threads);
        assert!(result.is_err());
    }

    // gpu_id tests
    #[test]
    fn test_validate_gpu_id_valid() {
        let id = "0";
        let result = Cli::validate_gpu_id(id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_validate_gpu_id_valid_multiple_digits() {
        let id = "3";
        let result = Cli::validate_gpu_id(id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_validate_gpu_id_invalid_too_high() {
        let id = "100";
        let result = Cli::validate_gpu_id(id);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_gpu_id_invalid_string() {
        let id = "abc";
        let result = Cli::validate_gpu_id(id);
        assert!(result.is_err());
    }

    // num_addresses tests
    #[test]
    fn test_validate_num_addresses_valid_positive() {
        let num = "10";
        let result = Cli::validate_num_addresses(num);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn test_validate_num_addresses_zero() {
        let num = "0";
        let result = Cli::validate_num_addresses(num);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_validate_num_addresses_one() {
        let num = "1";
        let result = Cli::validate_num_addresses(num);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_validate_num_addresses_negative() {
        let num = "-1";
        let result = Cli::validate_num_addresses(num);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_num_addresses_invalid_string() {
        let num = "abc";
        let result = Cli::validate_num_addresses(num);
        assert!(result.is_err());
    }

    // Tests for conflicting options validation
    #[test]
    fn test_validate_conflicting_options_gpu_only_with_cpu_threads() {
        let cli = Cli {
            prefixes: vec![Prefix::new("1A").unwrap()],
            xpub: "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn".to_string(),
            max_depth: 1000,
            cpu_threads: 4,
            gpu: None,
            gpu_only: true,
            num_addresses: 1,
        };

        let result = cli.validate_conflicting_options();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("--gpu-only and -t/--cpu-threads cannot be used together"));
    }

    #[test]
    fn test_validate_conflicting_options_gpu_only_with_zero_threads() {
        let cli = Cli {
            prefixes: vec![Prefix::new("1A").unwrap()],
            xpub: "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn".to_string(),
            max_depth: 1000,
            cpu_threads: 0, // 0 means auto-detect, which is valid with gpu_only
            gpu: None,
            gpu_only: true,
            num_addresses: 1,
        };

        let result = cli.validate_conflicting_options();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_conflicting_options_no_gpu_only_with_threads() {
        let cli = Cli {
            prefixes: vec![Prefix::new("1A").unwrap()],
            xpub: "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn".to_string(),
            max_depth: 1000,
            cpu_threads: 4,
            gpu: None,
            gpu_only: false,
            num_addresses: 1,
        };

        let result = cli.validate_conflicting_options();
        assert!(result.is_ok());
    }

    // Tests for multiple prefixes validation
    #[test]
    fn test_validate_multiple_prefixes_valid() {
        let cli = Cli {
            prefixes: vec![Prefix::new("1A").unwrap(), Prefix::new("1B").unwrap()],
            xpub: "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn".to_string(),
            max_depth: 1000,
            cpu_threads: 4,
            gpu: None,
            gpu_only: false,
            num_addresses: 1,
        };

        let result = cli.validate_conflicting_options();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_prefixes() {
        let cli = Cli {
            prefixes: vec![],
            xpub: "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn".to_string(),
            max_depth: 1000,
            cpu_threads: 4,
            gpu: None,
            gpu_only: false,
            num_addresses: 1,
        };

        let result = cli.validate_conflicting_options();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("At least one prefix must be provided"));
    }

    #[test]
    fn test_validate_too_many_prefixes() {
        let mut prefixes = Vec::new();
        for _ in 0..257 {
            // Generate 257 prefixes
            prefixes.push(Prefix::new("1A").unwrap());
        }

        let cli = Cli {
            prefixes,
            xpub: "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn".to_string(),
            max_depth: 1000,
            cpu_threads: 4,
            gpu: None,
            gpu_only: false,
            num_addresses: 1,
        };

        let result = cli.validate_conflicting_options();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Maximum 256 prefixes allowed"));
    }
}
