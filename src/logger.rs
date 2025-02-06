use std::thread;
use std::time::Duration;
use std::io::{self, Write};

pub struct Logger {
    serious_mode: bool,
    pub first_log_status: bool,
}

impl Logger {
    pub fn new(serious_mode: bool) -> Self {
        Logger {
            serious_mode,
            first_log_status: true,
        }
    }

    pub fn start(&self, prefix: String) {
        if self.serious_mode {
            println!("Starting for prefix: {}", prefix);
        } else {
            println!("👨‍🎨: Hmmm, \"{}\" you say?", prefix);
            thread::sleep(Duration::from_secs(2));
            println!("👨‍🎨: What an interesting prefix!");
            thread::sleep(Duration::from_secs(2));
            println!("👨‍🎨: Ok, lets do it!");
        }
    }
    pub fn wait_for_hashrate(&self, wait_time: u8) {
        if self.serious_mode {
            print!("Waiting {} seconds to reach initial hashrate", wait_time);
        } else {
            print!(
                "👨‍🎨: Just wait here for {} seconds, I will prepare the studio",
                wait_time
            );
        }
    }
    pub fn print_statistics(&self, hashrate: f64) {
        if self.serious_mode {
            println!("Initial hashrate: {:.2} addresses/s", hashrate);
        } else {
            println!(
                "👨‍🎨: I'm building around {:.0} addresses every second 😮‍💨",
                hashrate
            );
            thread::sleep(Duration::from_secs(1));
            println!("👨‍🎨: I hope to make one that you'll like...");
        }
    }
    pub fn log_status(&mut self, generated: usize, found: usize, run_time: f64, hashrate: f64) {
        if self.first_log_status {
            self.first_log_status = false;
            println!("");
        }

        if self.serious_mode {
            self.erase_previous_line();
            println!("[{:.2} add/s] {} in {:.1} seconds", hashrate, generated, run_time);
        } else {
            self.erase_previous_line();
            if found == 0 {
                println!(
                    "👨‍🎨: I already built {} addresses in the last {:.0} seconds. Wow, that's {:.0} per second!",
                    generated, run_time, hashrate
                );
            } else {
                println!(
                    "I already built {} addresses in the last {} seconds. Wow, that is {} per second! I found {} addresses!",
                    generated, run_time, hashrate, found
                );
            }
        }
    }
    fn erase_previous_line(&self) {
        print!("\x1B[1A\x1B[2K"); // Move up one line and clear it
        io::stdout().flush().unwrap();
    }
}
