mod cli;

fn main() {
    let args = cli::Cli::parse_args();
    let pubkey_hash: [u8; 20] = args.pubkey_hash;

    println!("🎨 Address Artisan is starting!");
    println!("Using pubkey hash: {:?}", pubkey_hash);
}
