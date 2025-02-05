use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct StateHandler {
    generated_local: usize,
    global_generated_counter: Arc<AtomicUsize>,
    global_found_counter: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
    local_batch_size: usize,
    start_time: Instant,
}

impl StateHandler {
    pub fn new(
        global_generated_counter: Arc<AtomicUsize>,
        global_found_counter: Arc<AtomicUsize>,
        running: Arc<AtomicBool>,
        local_batch_size: usize,
    ) -> Self {
        Self {
            generated_local: 0,
            global_generated_counter,
            global_found_counter,
            running,
            local_batch_size,
            start_time: Instant::now(),
        }
    }

    pub fn new_generated(&mut self) {
        self.generated_local += 1;
        if self.generated_local >= self.local_batch_size {
            self.flush_generated();
        }
    }

    fn flush_generated(&mut self) {
        self.global_generated_counter
            .fetch_add(self.generated_local, Ordering::Relaxed);
        self.generated_local = 0;
    }

    pub fn new_found(&mut self) {
        self.flush_generated();
        self.global_found_counter.fetch_add(1, Ordering::Relaxed);
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn get_generated(&self) -> usize {
        self.global_generated_counter.load(Ordering::Relaxed)
    }

    pub fn get_found(&self) -> usize {
        self.global_found_counter.load(Ordering::Relaxed)
    }

    pub fn get_run_time(&self) -> Duration {
        let start_time = self.start_time;
        let current_time = Instant::now();
        current_time.duration_since(start_time)
    }

    pub fn get_hashrate(&self) -> f64 {
        let total_generated = self.get_generated();
        let run_time = self.get_run_time();
        let total_rate = total_generated as f64 / run_time.as_secs_f64();
        total_rate
    }

    pub fn get_statistics(&self) -> (usize, usize, f64, f64) {
        let total_generated = self.get_generated();
        let total_found = self.get_found();
        let run_time = self.get_run_time().as_secs_f64();
        let hashrate = total_generated as f64 / run_time;
        (total_generated, total_found, run_time, hashrate)
    }
}
