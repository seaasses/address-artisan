mod bitcoin_address_helper;
mod cli;
mod prefix_validator;
mod stats_logger;
mod vanity_address_finder;
mod xpub;

use cli::Cli;
use rand;
use stats_logger::StatsLogger;
use std::sync::{mpsc, Arc};
use std::thread;
use vanity_address_finder::VanityAddressFinder;

fn main() {
    let cli = Cli::parse_args();
    let num_threads = thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4);
    println!(
        "Starting vanity address search with {} threads...",
        num_threads
    );

    // Create shared stats logger and channel for result
    let stats_logger = Arc::new(StatsLogger::new());
    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    // Spawn threads
    for _ in 0..num_threads {
        let prefix = cli.prefix.clone();
        let xpub = cli.xpub.clone();
        let stats_logger = Arc::clone(&stats_logger);
        let tx = tx.clone();

        let handle = thread::spawn(move || {
            // Create a path starting with a random number and the thread's offset
            let start_path = vec![rand::random::<u32>() & 0x7FFFFFFF];
            let mut finder =
                VanityAddressFinder::new(prefix, xpub, cli.max_depth, stats_logger, start_path);

            // Search with a step size equal to the number of threads
            if let Some(result) = finder.find_address() {
                let _ = tx.send(result);
            }
        });

        handles.push(handle);
    }

    drop(tx);

    let result = rx.recv().ok();

    for handle in handles {
        let _ = handle.join();
    }

    let (total_generated, total_found, final_rate) = stats_logger.get_stats();
    stats_logger.stop();

    match result {
        Some((address, path)) => {
            println!("\nSuccess! Final statistics:");
            println!("Total addresses generated: {}", total_generated);
            println!("Total matching addresses found: {}", total_found);
            println!("Final generation rate: {:.2} addr/s", final_rate);
            println!("\nFound address: {}. Path: xpub/{}", address, path);
        }
        None => {
            println!("\nNo matching address found. Final statistics:");
            println!("Total addresses generated: {}", total_generated);
            println!("Final generation rate: {:.2} addr/s", final_rate);
        }
    }
}
