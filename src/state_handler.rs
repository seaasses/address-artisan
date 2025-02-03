use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct StateHandler {
    generated_local: usize,
    global_generated_counter: Arc<AtomicUsize>,
    global_found_counter: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
    local_batch_size: usize,
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
        }
    }

    pub fn new_generated(&mut self) {
        self.generated_local += 1;
        if self.generated_local == self.local_batch_size {
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
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}
