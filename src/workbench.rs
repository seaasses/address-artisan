pub trait Workbench: Send + Sync {
    fn start(&self);
    fn wait(&self);
    fn total_generated(&self) -> u64;
}
