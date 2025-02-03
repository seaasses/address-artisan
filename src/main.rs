mod bitcoin_address_helper;
mod cli;
mod vanity_address;
mod xpub;
mod xpub_path_walker;

use cli::Cli;
use rand;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use vanity_address::VanityAddress;
use xpub_path_walker::XpubPubkeyHashWalker;

const STATUS_UPDATE_INTERVAL: Duration = Duration::from_secs(5);
const THREADS_BATCH_SIZE: usize = 10000;

fn main() {
    let cli = Cli::parse_args();
    let num_threads = thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4);
    println!(
        "Starting vanity address search with {} threads...",
        num_threads
    );

    let mut handles = Vec::with_capacity(num_threads);
    let prefix = cli.prefix.clone();
    let max_depth = cli.max_depth;
    let counter = Arc::new(AtomicUsize::new(0));
    let running = Arc::new(AtomicBool::new(true));
    let start_time = Instant::now();

    // Spawn status update thread
    let status_counter = Arc::clone(&counter);
    let status_running = Arc::clone(&running);
    let status_handle = thread::spawn(move || {
        while status_running.load(Ordering::Relaxed) {
            thread::sleep(STATUS_UPDATE_INTERVAL);
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
    for _ in 0..num_threads {
        let xpub = cli.xpub.clone();
        let prefix = prefix.clone();
        let counter = Arc::clone(&counter);
        let running = Arc::clone(&running);

        let handle = thread::spawn(move || {
            let vanity_address = VanityAddress::new(&prefix);
            let initial_path = vec![rand::random::<u32>() & 0x7FFFFFFF];
            let walker = XpubPubkeyHashWalker::new(xpub, initial_path, max_depth);

            let mut local_counter = 0;

            for pubkey_hash in walker {
                if !running.load(Ordering::Relaxed) {
                    return;
                }

                if let Some(address) = vanity_address.get_vanity_address(pubkey_hash) {
                    println!("Found address: {}", address);
                    running.store(false, Ordering::Relaxed);
                    counter.fetch_add(local_counter, Ordering::Relaxed);
                    return;
                }

                local_counter += 1;
                if local_counter >= THREADS_BATCH_SIZE {
                    counter.fetch_add(local_counter, Ordering::Relaxed);
                    local_counter = 0;
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Signal status thread to stop and wait for it
    running.store(false, Ordering::Relaxed);
    status_handle.join().unwrap();

    // Print final statistics
    let total_checked = counter.load(Ordering::Relaxed);
    let total_time = start_time.elapsed().as_secs_f64();
    let final_rate = total_checked as f64 / total_time;
    println!("\nSearch completed!");
    println!("Total addresses checked: {}", total_checked);
    println!("Total time: {:.2} seconds", total_time);
    println!("Average rate: {:.2} addresses/second", final_rate);
}
