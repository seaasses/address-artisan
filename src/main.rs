mod bitcoin_address_helper;
mod cli;
mod prefix_validator;
mod xpub;

fn main() {
    let args = cli::Cli::parse_args();
    let prefix_validator = prefix_validator::PrefixValidator::new("1Nv".to_string());
    let xpub = "xpub6DEdcQRGMADjaUJC7mf9C1zE7nYSyYAoxAuP72z4unDV6d2E4scTWQgUxv9Cx5PkmkxJ2P3HFWScnwjVAiUfFY3RuLGcDsk7dsSYDKwyrA7";
    let xpub_wrapper = xpub::XpubWrapper::new(xpub);
    let bitcoin_address_helper = bitcoin_address_helper::BitcoinAddressHelper::new();

    let mut found = false;
    for i in 0..100000 {
        let pubkey_hash = xpub_wrapper.get_pubkey_hash_160(vec![0, 8, 0, i]);
        let unwrapped_hash = pubkey_hash.unwrap();
        let is_valid = prefix_validator.is_valid(unwrapped_hash);
        if is_valid {
            println!("Found valid address: 0/ {}", i);
            let address = bitcoin_address_helper.get_address_from_pubkey_hash(unwrapped_hash);
            println!("Address: {}", address);
            found = true;
            break;
        }
    }
    if !found {
        println!("No valid address found");
    }
}
