mod bitcoin_address_helper;
mod cli;
mod prefix_validator;
mod vanity_address_finder;
mod xpub;
mod stats_logger;

use cli::Cli;
use rand;
use vanity_address_finder::VanityAddressFinder;

fn main() {
    let cli = Cli::parse_args();
    let mut address_finder = VanityAddressFinder::new(
        cli.prefix,
        cli.xpub,
        vec![rand::random::<u32>() & 0x7FFFFFFF],
        cli.max_depth,
    );

    println!("Starting vanity address search...");
    let result = address_finder.find_address();

    // Get a reference to the stats logger and stop it
    let stats = address_finder.get_stats_logger();
    let (total_generated, total_found, final_rate) = stats.get_stats();
    stats.stop();

    match result {
        Ok((address, path)) => {
            println!("\nSuccess! Final statistics:");
            println!("Total addresses generated: {}", total_generated);
            println!("Total matching addresses found: {}", total_found);
            println!("Final generation rate: {:.2} addr/s", final_rate);
            println!("\nFound address: {}. Path: xpub/{}", address, path);
        }
        Err(e) => eprintln!("Error finding address: {}", e),
    }
}
