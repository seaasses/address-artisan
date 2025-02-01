mod bitcoin_address_helper;
mod cli;
mod prefix_validator;
mod vanity_address_finder;
mod xpub;

fn main() {
    let args = cli::Cli::parse_args();
    let xpub = "xpub6DEdcQRGMADjaUJC7mf9C1zE7nYSyYAoxAuP72z4unDV6d2E4scTWQgUxv9Cx5PkmkxJ2P3HFWScnwjVAiUfFY3RuLGcDsk7dsSYDKwyrA7";

    let vanity_address_finder = vanity_address_finder::VanityAddressFinder::new(
        "1Mr".to_string(),
        xpub.to_string(),
        vec![0],
    );
    match vanity_address_finder.find_address() {
        Ok((address, path)) => println!("Found address: {}. Path: xpub/{}", address, path),
        Err(e) => eprintln!("Error finding address: {}", e),
    }
}
