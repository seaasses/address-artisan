mod bitcoin_address_helper;
mod cli;
mod extended_public_key_deriver;
mod extended_public_key_path_walker;
mod state_handler;
mod vanity_address;

use cli::Cli;
use extended_public_key_deriver::ExtendedPublicKeyDeriver;
use extended_public_key_path_walker::ExtendedPublicKeyPathWalker;
use rand;
use std::thread;
use std::time::{Duration, Instant};
use vanity_address::VanityAddress;

const STATUS_UPDATE_INTERVAL: Duration = Duration::from_secs(2);
const THREADS_BATCH_SIZE: usize = 1000;
const WAIT_TIME_FOR_INITIAL_HASHRATE: u8 = 5;

fn main() -> Result<(), String> {
    let cli = Cli::parse_args();
    let num_threads = thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4);
    println!(
        "Starting vanity address search with {} threads...",
        num_threads
    );
    let max_depth = cli.max_depth;
    let xpub = cli.xpub;
    let prefix = cli.prefix;

    let xpub_deriver = ExtendedPublicKeyDeriver::new(&xpub)?;

    let init = Instant::now();
    let n = 400000;
    let initial_path = vec![rand::random::<u32>() & 0x7FFFFFFF];
    let vanity_address = VanityAddress::new(&prefix);

    let mut xpub_path_walker = ExtendedPublicKeyPathWalker::new(initial_path, max_depth);
    for _ in 0..10 {
        let init = Instant::now();
        let xpaths = xpub_path_walker.get_n_next_paths(n);
        let fdjinal = Instant::now();
        println!(
            "Time taken to get {} paths: {:?}",
            n,
            fdjinal.duration_since(init)
        );
        let pubkey_hashes = xpub_deriver.get_pubkeys_hash_160(&xpaths)?;
        let init_with_pubkeys = Instant::now();
        for (i, pubkey_hash) in pubkey_hashes.iter().enumerate() {
            match vanity_address.get_vanity_address(*pubkey_hash) {
                Some(address) => {
                    println!("Found vanity address {} at path {:?}", address, xpaths[i]);
                }
                None => {}
            }
        }
        let finished_with_pubkeys = Instant::now();
        println!(
            "Time taken to test {} pubkeys: {:?}",
            n,
            finished_with_pubkeys.duration_since(init_with_pubkeys)
        );
    }

    let finished = Instant::now();
    println!("Time taken: {:?}", finished.duration_since(init));
    Ok(())
}
