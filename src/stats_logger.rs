use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

pub struct StatsLogger {
    start_time: Instant,
    addresses_generated: Arc<AtomicU64>,
    addresses_found: Arc<AtomicU64>,
    is_running: Arc<AtomicBool>,
    should_stop: Arc<AtomicBool>,
}

impl StatsLogger {
    pub fn new() -> Self {
        let logger = StatsLogger {
            start_time: Instant::now(),
            addresses_generated: Arc::new(AtomicU64::new(0)),
            addresses_found: Arc::new(AtomicU64::new(0)),
            is_running: Arc::new(AtomicBool::new(true)),
            should_stop: Arc::new(AtomicBool::new(false)),
        };

        // Start background logging thread
        let generated = Arc::clone(&logger.addresses_generated);
        let found = Arc::clone(&logger.addresses_found);
        let is_running = Arc::clone(&logger.is_running);
        let start = logger.start_time;

        thread::spawn(move || {
            let mut stdout = io::stdout();
            while is_running.load(Ordering::Relaxed) {
                let generated_count = generated.load(Ordering::Relaxed);
                let found_count = found.load(Ordering::Relaxed);
                let elapsed = start.elapsed();
                let rate = generated_count as f64 / elapsed.as_secs_f64();

                // Use write! instead of println! for buffered output
                let _ = write!(
                    stdout,
                    "\rStats: Generated {} addresses, Found {}, Rate: {:.2} addr/s",
                    generated_count, found_count, rate
                );
                let _ = stdout.flush();

                thread::sleep(std::time::Duration::from_secs(5));
            }
            println!(); // Print newline when done
        });

        logger
    }

    pub fn increment_generated(&self) {
        self.addresses_generated.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_found(&self) {
        self.addresses_found.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> (u64, u64, f64) {
        let generated = self.addresses_generated.load(Ordering::Relaxed);
        let found = self.addresses_found.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed();
        let rate = generated as f64 / elapsed.as_secs_f64();
        (generated, found, rate)
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }

    pub fn should_stop(&self) -> bool {
        self.should_stop.load(Ordering::Relaxed)
    }

    pub fn signal_stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }
}