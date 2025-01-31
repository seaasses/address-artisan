use clap::Parser;

#[derive(Parser)]
#[command(
    version,
    about = "Vanity P2PKH Bitcoin Address Generator",
    long_about = "A tool for generating vanity P2PKH Bitcoin addresses."
)]

pub struct Cli {
    #[arg(
        short='P',
        long = "pubkey-hash",
        help = "Bitcoin public key hash (20 bytes in hex)",
        value_parser = Cli::validate_pubkey_hash
    )]
    pub pubkey_hash: [u8; 20],

    #[arg(short = 'p', long = "prefix", help = "Prefix for the address", value_parser = Cli::validate_prefix)]
    pub prefix: String,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
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

    fn validate_pubkey_hash(pubkey_hash: &str) -> Result<[u8; 20], String> {
        if pubkey_hash.is_empty() {
            return Err("Pubkey hash cannot be empty".to_string());
        }

        if !pubkey_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Pubkey hash must contain only hexadecimal characters".to_string());
        }

        if pubkey_hash.len() != 40 {
            // 20 bytes = 40 hex characters
            return Err("Pubkey hash must be 20 bytes (40 hex characters) long".to_string());
        }

        let mut pubkey_hash_bytes = [0u8; 20];
        for i in 0..20 {
            let byte_str = &pubkey_hash[i * 2..i * 2 + 2];
            pubkey_hash_bytes[i] =
                u8::from_str_radix(byte_str, 16).map_err(|_| "Invalid hex value".to_string())?;
        }
        Ok(pubkey_hash_bytes)
    }
}
