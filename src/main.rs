mod bitcoin_address_helper;
mod cli;
mod prefix_validator;
mod vanity_address_finder;
mod xpub;
use rand;

fn main() {
    let args = cli::Cli::parse_args();
    let xpub = args.xpub;
    let prefix = args.prefix;
    let max_depth = args.max_depth;

    let initial_path = vec![rand::random::<u32>() & 0x7FFFFFFF];
    let address_finder =
        vanity_address_finder::VanityAddressFinder::new(prefix, xpub, initial_path, max_depth);

    match address_finder.find_address() {
        Ok((address, path)) => println!("Found address: {}. Path: xpub/{}", address, path),
        Err(e) => eprintln!("Error finding address: {}", e),
    }
}
