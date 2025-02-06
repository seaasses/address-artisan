mod bitcoin_address_helper;
mod cli;
mod extended_public_key_deriver;
mod extended_public_key_path_walker;
mod logger;
mod state_handler;
mod vanity_address;

use cli::Cli;
use extended_public_key_deriver::ExtendedPublicKeyDeriver;
use extended_public_key_path_walker::ExtendedPublicKeyPathWalker;
use logger::Logger;
use rand;
use state_handler::StateHandler;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use vanity_address::VanityAddress;

const STATUS_UPDATE_INTERVAL: Duration = Duration::from_secs(3);
const THREADS_BATCH_SIZE: usize = 400000;
const WAIT_TIME_FOR_INITIAL_HASHRATE: u8 = 30;

fn setup_worker_thread(
    xpub: String,
    prefix: String,
    max_depth: u32,
    global_generated_counter: Arc<AtomicUsize>,
    global_found_counter: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
    found_addresses: Arc<Mutex<Vec<(String, Vec<u32>)>>>,
) -> Result<(), String> {
    let initial_path = vec![rand::random::<u32>() & 0x7FFFFFFF];
    let mut xpub_path_walker = ExtendedPublicKeyPathWalker::new(initial_path, max_depth);
    let xpub_deriver = ExtendedPublicKeyDeriver::new(&xpub)?;
    let vanity_address = VanityAddress::new(&prefix);
    let mut state_handler = StateHandler::new(
        Arc::clone(&global_generated_counter),
        Arc::clone(&global_found_counter),
        running,
        THREADS_BATCH_SIZE,
        Arc::clone(&found_addresses),
    );

    while state_handler.is_running() {
        let xpaths = xpub_path_walker.get_n_next_paths(THREADS_BATCH_SIZE);
        let pubkey_hashes = xpub_deriver.get_pubkeys_hash_160(&xpaths)?;

        for (i, pubkey_hash) in pubkey_hashes.iter().enumerate() {
            if !state_handler.is_running() {
                return Ok(());
            }
            state_handler.new_generated();

            match vanity_address.get_vanity_address(*pubkey_hash) {
                Some(address) => {
                    let xpath_path = xpaths[i].clone();
                    state_handler.add_found_address(address, xpath_path);
                }
                None => {}
            }
        }
    }
    Ok(())
}

fn setup_logger_thread(
    global_generated_counter: Arc<AtomicUsize>,
    global_found_counter: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
    prefix: String,
    found_addresses: Arc<Mutex<Vec<(String, Vec<u32>)>>>,
    serious_mode: bool,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut logger = Logger::new(serious_mode);

        let state_handler = StateHandler::new(
            Arc::clone(&global_generated_counter),
            Arc::clone(&global_found_counter),
            running,
            THREADS_BATCH_SIZE,
            Arc::clone(&found_addresses),
        );

        if !state_handler.is_running() {
            return;
        }
        logger.start(prefix);

        if !state_handler.is_running() {
            let found_addresses = state_handler.get_found_addresses();
            for (address, xpath_path) in found_addresses {
                logger.log_found_address(&address, &xpath_path);
            }
            return;
        }
        logger.wait_for_hashrate(WAIT_TIME_FOR_INITIAL_HASHRATE);

        thread::sleep(Duration::from_secs(2));
        for _ in 0..WAIT_TIME_FOR_INITIAL_HASHRATE {
            thread::sleep(Duration::from_secs(1));
            if !state_handler.is_running() {
                break;
            }
            print!(".");
            io::stdout().flush().unwrap();
        }
        println!();

        if state_handler.is_running() {
            let hashrate = state_handler.get_hashrate();
            logger.print_statistics(hashrate);
        }

        while state_handler.is_running() {
            let (generated, found, run_time, hashrate) = state_handler.get_statistics();
            logger.log_status(generated, found, run_time, hashrate);

            thread::sleep(STATUS_UPDATE_INTERVAL);
        }

        if state_handler.get_found() > 0 {
            let found_addresses = state_handler.get_found_addresses();
            for (address, xpath_path) in found_addresses {
                logger.log_found_address(&address, &xpath_path);
            }
        }
    })
}

fn main() {
    let cli = Cli::parse_args();
    let global_generated_counter = Arc::new(AtomicUsize::new(0));
    let global_found_counter = Arc::new(AtomicUsize::new(0));
    let running = Arc::new(AtomicBool::new(true));
    let found_addresses = Arc::new(Mutex::new(Vec::new()));
    let serious_mode = !cli.i_am_boring;

    // Spawn logger thread
    let logger_handle = setup_logger_thread(
        Arc::clone(&global_generated_counter),
        Arc::clone(&global_found_counter),
        Arc::clone(&running),
        cli.prefix.clone(),
        Arc::clone(&found_addresses),
        serious_mode,
    );

    // Spawn single worker thread
    let worker_handle = {
        let thread_running = Arc::clone(&running);
        thread::spawn(move || {
            if let Err(e) = setup_worker_thread(
                cli.xpub,
                cli.prefix,
                cli.max_depth,
                global_generated_counter,
                global_found_counter,
                thread_running,
                found_addresses,
            ) {
                eprintln!("Worker thread error: {}", e);
            }
        })
    };

    // Wait for worker to finish
    worker_handle.join().unwrap();
    logger_handle.join().unwrap();
}
