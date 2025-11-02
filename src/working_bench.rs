use std::time::Duration;

pub trait WorkingBench: Send {
    fn start(&mut self);
    fn stop(&mut self);
    fn wait(self);
    fn get_stats(&self) -> BenchStats;
}

pub struct BenchStats {
    pub bench_id: String,
    pub addresses_generated: u64,
    pub addresses_found: u64,
    pub elapsed_time: Duration,
}
