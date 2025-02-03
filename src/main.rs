mod bitcoin_address_helper;
mod cli;
mod extended_public_key_deriver;
mod extended_public_key_path_walker;
mod state_handler;
mod vanity_address;

use cli::Cli;
use extended_public_key_deriver::ExtendedPublicKeyDeriver;
use extended_public_key_path_walker::ExtendedPUblicKeyPathWalker;
use rand;
use state_handler::StateHandler;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use vanity_address::VanityAddress;

const STATUS_UPDATE_INTERVAL: Duration = Duration::from_secs(2);
const THREADS_BATCH_SIZE: usize = 1000;
const WAIT_TIME_FOR_INITIAL_HASHRATE: u8 = 5;

fn setup_worker_thread(
    xpub: String,
    prefix: String,
    max_depth: u32,
    global_generated_counter: Arc<AtomicUsize>,
    global_found_counter: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
) {
    let initial_path = vec![rand::random::<u32>() & 0x7FFFFFFF];
    let xpub_path_walker = ExtendedPUblicKeyPathWalker::new(initial_path, max_depth);
    let mut xpub_deriver = ExtendedPublicKeyDeriver::new(&xpub);
    let vanity_address = VanityAddress::new(&prefix);
    let mut state_handler = StateHandler::new(
        Arc::clone(&global_generated_counter),
        Arc::clone(&global_found_counter),
        running,
        THREADS_BATCH_SIZE,
    );

    for xpub_path in xpub_path_walker {
        if !state_handler.is_running() {
            return;
        }
        state_handler.new_generated();
        let pubkey_hash = xpub_deriver.get_pubkey_hash_160(&xpub_path).unwrap();
        match vanity_address.get_vanity_address(pubkey_hash) {
            Some(address) => {
                let xpath_path_string = xpub_path
                    .iter()
                    .take(xpub_path.len().saturating_sub(2))
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>()
                    .join("/");
                let receive_address = xpub_path[xpub_path.len() - 1];
                println!(
                    "Found address: {} at xpub/{}, receive address {}",
                    address, xpath_path_string, receive_address
                );
                state_handler.new_found();
                return;
            }
            None => {}
        }
    }

    state_handler.new_generated();
}

fn setup_logger_thread(
    global_generated_counter: Arc<AtomicUsize>,
    global_found_counter: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let state_handler = StateHandler::new(
            Arc::clone(&global_generated_counter),
            Arc::clone(&global_found_counter),
            running,
            THREADS_BATCH_SIZE,
        );

        for _ in 0..WAIT_TIME_FOR_INITIAL_HASHRATE {
            thread::sleep(Duration::from_secs(1));
            if !state_handler.is_running() {
                return;
            }
        }
        let hashrate = state_handler.get_hashrate();
        println!("INITIAL HASHRATE");
        println!("{:.2} addresses/s", hashrate);

        while state_handler.is_running() {
            let (generated, found, run_time, hashrate) = state_handler.get_statistics();
            println!(
                "{} addresses generated, {} addresses found, {:.0} seconds, {:.2} addresses/s",
                generated, found, run_time, hashrate
            );
            thread::sleep(STATUS_UPDATE_INTERVAL);
        }
    })
}

fn setup_worker_threads(
    xpub: String,
    prefix: String,
    max_depth: u32,
    num_threads: usize,
    global_generated_counter: Arc<AtomicUsize>,
    global_found_counter: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
) -> Vec<thread::JoinHandle<()>> {
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::with_capacity(num_threads);
    for _ in 0..num_threads {
        let thread_xpub = xpub.clone();
        let thread_prefix = prefix.clone();
        let thread_max_depth = max_depth;
        let thread_global_generated_counter = Arc::clone(&global_generated_counter);
        let thread_global_found_counter = Arc::clone(&global_found_counter);
        let thread_running = Arc::clone(&running);

        let handle = thread::spawn(move || {
            setup_worker_thread(
                thread_xpub,
                thread_prefix,
                thread_max_depth,
                thread_global_generated_counter,
                thread_global_found_counter,
                thread_running,
            );
        });
        handles.push(handle);
    }
    handles
}

fn setup_threads(
    xpub: String,
    prefix: String,
    max_depth: u32,
    num_threads: usize,
) -> (thread::JoinHandle<()>, Vec<thread::JoinHandle<()>>) {
    let global_generated_counter = Arc::new(AtomicUsize::new(0));
    let global_found_counter = Arc::new(AtomicUsize::new(0));
    let running = Arc::new(AtomicBool::new(true));

    // Spawn status update thread
    let status_handle = setup_logger_thread(
        Arc::clone(&global_generated_counter),
        Arc::clone(&global_found_counter),
        Arc::clone(&running),
    );

    // Spawn worker threads
    let worker_handles = setup_worker_threads(
        xpub,
        prefix,
        max_depth,
        num_threads,
        Arc::clone(&global_generated_counter),
        Arc::clone(&global_found_counter),
        Arc::clone(&running),
    );

    (status_handle, worker_handles)
}

fn main() {
    let cli = Cli::parse_args();
    let num_threads = thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4);
    println!(
        "Starting vanity address search with {} threads...",
        num_threads
    );

    let (logger_handle, worker_handles) =
        setup_threads(cli.xpub, cli.prefix, cli.max_depth, num_threads);

    for handle in worker_handles {
        handle.join().unwrap();
    }

    logger_handle.join().unwrap();
}
