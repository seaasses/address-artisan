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
        let max_depth: u32 = max_depth
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;

        if max_depth > 0x7FFFFFFF {
            return Err("Max depth must be less than 2^31 - 1".to_string());
        }

        Ok(max_depth)
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
