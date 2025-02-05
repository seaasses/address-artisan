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
use state_handler::StateHandler;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use vanity_address::VanityAddress;

const STATUS_UPDATE_INTERVAL: Duration = Duration::from_secs(2);
const THREADS_BATCH_SIZE: usize = 400000;
const WAIT_TIME_FOR_INITIAL_HASHRATE: u8 = 30;

fn setup_worker_thread(
    xpub: String,
    prefix: String,
    max_depth: u32,
    global_generated_counter: Arc<AtomicUsize>,
    global_found_counter: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
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
                    let xpath_path_string = xpaths[i]
                        .iter()
                        .take(xpaths[i].len().saturating_sub(2))
                        .map(|p| p.to_string())
                        .collect::<Vec<String>>()
                        .join("/");
                    let receive_address = xpaths[i][xpaths[i].len() - 1];
                    println!(
                        "Found address: {} at xpub/{}, receive address {}",
                        address, xpath_path_string, receive_address
                    );
                    state_handler.new_found();
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
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let state_handler = StateHandler::new(
            Arc::clone(&global_generated_counter),
            Arc::clone(&global_found_counter),
            running,
            THREADS_BATCH_SIZE,
        );

        let mut wait_time = WAIT_TIME_FOR_INITIAL_HASHRATE;
        let time_first_message = Duration::from_secs(2);
        let time_second_message = Duration::from_secs(2);
        let time_third_message = Duration::from_secs(1);
        let time_fourth_message = Duration::from_secs(1);

        // First message
        if !state_handler.is_running() {
            return;
        }
        println!("👨‍🎨: Hmmm, \"{}\" you say?", prefix);
        thread::sleep(time_first_message);
        wait_time -= time_first_message.as_secs() as u8;

        if !state_handler.is_running() {
            return;
        }
        // Second message
        println!("👨‍🎨: What an interesting prefix!");
        thread::sleep(time_second_message);
        wait_time -= time_second_message.as_secs() as u8;
        if !state_handler.is_running() {
            return;
        }

        // third message
        println!("👨‍🎨: Ok, lets do it!");
        thread::sleep(time_third_message);
        wait_time -= time_third_message.as_secs() as u8;

        if !state_handler.is_running() {
            return;
        }

        // fourth message
        print!(
            "👨‍🎨: Just wait here for {} seconds, I will prepare the studio",
            wait_time
        );
        thread::sleep(time_fourth_message);

        for _ in 0..wait_time {
            thread::sleep(Duration::from_secs(1));
            if !state_handler.is_running() {
                return;
            }
            print!(".");
            io::stdout().flush().unwrap();
        }
        println!();
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

fn main() {
    let cli = Cli::parse_args();
    let global_generated_counter = Arc::new(AtomicUsize::new(0));
    let global_found_counter = Arc::new(AtomicUsize::new(0));
    let running = Arc::new(AtomicBool::new(true));

    // Spawn logger thread
    let logger_handle = setup_logger_thread(
        Arc::clone(&global_generated_counter),
        Arc::clone(&global_found_counter),
        Arc::clone(&running),
        cli.prefix.clone(),
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
            ) {
                eprintln!("Worker thread error: {}", e);
            }
        })
    };

    // Wait for worker to finish
    worker_handle.join().unwrap();
    logger_handle.join().unwrap();
}
