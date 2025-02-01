use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct StatsLogger {
    start_time: Instant,
    addresses_generated: Arc<Mutex<u64>>,
    addresses_found: Arc<Mutex<u64>>,
    is_running: Arc<Mutex<bool>>,
    should_stop: Arc<AtomicBool>,
}

impl StatsLogger {
    pub fn new() -> Self {
        let logger = StatsLogger {
            start_time: Instant::now(),
            addresses_generated: Arc::new(Mutex::new(0)),
            addresses_found: Arc::new(Mutex::new(0)),
            is_running: Arc::new(Mutex::new(true)),
            should_stop: Arc::new(AtomicBool::new(false)),
        };

        // Start background logging thread
        let generated = Arc::clone(&logger.addresses_generated);
        let found = Arc::clone(&logger.addresses_found);
        let is_running = Arc::clone(&logger.is_running);
        let start = logger.start_time;

        thread::spawn(move || {
            while *is_running.lock().unwrap() {
                let generated_count = *generated.lock().unwrap();
                let found_count = *found.lock().unwrap();
                let elapsed = start.elapsed();
                let rate = generated_count as f64 / elapsed.as_secs_f64();

                println!(
                    "Stats: Generated {} addresses, Found {}, Rate: {:.2} addr/s",
                    generated_count, found_count, rate
                );

                thread::sleep(std::time::Duration::from_secs(1));
            }
        });

        logger
    }

    pub fn increment_generated(&self) {
        let mut count = self.addresses_generated.lock().unwrap();
        *count += 1;
    }

    pub fn increment_found(&self) {
        let mut count = self.addresses_found.lock().unwrap();
        *count += 1;
    }

    pub fn get_stats(&self) -> (u64, u64, f64) {
        let generated = *self.addresses_generated.lock().unwrap();
        let found = *self.addresses_found.lock().unwrap();
        let elapsed = self.start_time.elapsed();
        let rate = generated as f64 / elapsed.as_secs_f64();
        (generated, found, rate)
    }

    pub fn stop(&self) {
        let mut is_running = self.is_running.lock().unwrap();
        *is_running = false;
    }

    pub fn should_stop(&self) -> bool {
        self.should_stop.load(Ordering::Relaxed)
    }

    pub fn signal_stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }
}