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
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use vanity_address::VanityAddress;

const STATUS_UPDATE_INTERVAL: Duration = Duration::from_secs(2);
const THREADS_BATCH_SIZE: usize = 1000;

fn setup_worker_thread(
    xpub: String,
    prefix: String,
    max_depth: u32,
    counter: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
) {
    let initial_path = vec![rand::random::<u32>() & 0x7FFFFFFF];
    let xpub_path_walker = ExtendedPUblicKeyPathWalker::new(initial_path, max_depth);
    let mut xpub_deriver = ExtendedPublicKeyDeriver::new(&xpub);
    let vanity_address = VanityAddress::new(&prefix);
    let mut state_handler = StateHandler::new(
        Arc::clone(&counter),
        Arc::clone(&counter),
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

fn setup_threads(
    xpub: String,
    prefix: String,
    max_depth: u32,
    num_threads: usize,
) -> (
    Vec<thread::JoinHandle<()>>,
    Arc<AtomicUsize>,
    Arc<AtomicBool>,
    Instant,
    thread::JoinHandle<()>,
) {
    let counter = Arc::new(AtomicUsize::new(0));
    let running = Arc::new(AtomicBool::new(true));
    let start_time = Instant::now();

    // Spawn status update thread
    let status_counter = Arc::clone(&counter);
    let status_running = Arc::clone(&running);
    let status_handle = thread::spawn(move || {
        while status_running.load(Ordering::Relaxed) {
            thread::sleep(STATUS_UPDATE_INTERVAL);
            if !status_running.load(Ordering::Relaxed) {
                break;
            }
            let current_count = status_counter.load(Ordering::Relaxed);
            let current_time = Instant::now();

            let total_rate =
                current_count as f64 / current_time.duration_since(start_time).as_secs_f64();

            println!(
                "Checked {} addresses ({:.2} addresses/s)",
                current_count, total_rate
            );
        }
    });

    // Spawn worker threads
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::with_capacity(num_threads);
    for _ in 0..num_threads {
        let thread_xpub = xpub.clone();
        let thread_prefix = prefix.clone();
        let thread_max_depth = max_depth;
        let thread_counter = Arc::clone(&counter);
        let thread_running = Arc::clone(&running);

        let handle = thread::spawn(move || {
            setup_worker_thread(
                thread_xpub,
                thread_prefix,
                thread_max_depth,
                thread_counter,
                thread_running,
            );
        });
        handles.push(handle);
    }

    (handles, counter, running, start_time, status_handle)
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

    let (handles, _counter, _running, _start_time, status_handle) =
        setup_threads(cli.xpub, cli.prefix, cli.max_depth, num_threads);

    for handle in handles {
        handle.join().unwrap();
    }

    status_handle.join().unwrap();
}
