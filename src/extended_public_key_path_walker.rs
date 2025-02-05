pub struct ExtendedPublicKeyPathWalker {
    xpub_path: Vec<u32>,
    max_depth: u32,
    first_call: bool,
    max_non_hardening_index: u32,
}

impl ExtendedPublicKeyPathWalker {
    pub fn new(initial_path: Vec<u32>, max_depth: u32) -> Self {
        for derivation in initial_path.clone() {
            if derivation > 0x7FFFFFFF {
                panic!("Derivation index is greater than the max index");
            }
        }
        let xpub_path = [initial_path, vec![0, 0, 0]].concat();
        Self {
            xpub_path,
            max_depth,
            first_call: true,
            max_non_hardening_index: 0x7FFFFFFF,
        }
    }
    pub fn get_n_next_paths(&mut self, n: usize) -> Vec<Vec<u32>> {
        let mut paths = Vec::new();
        let initial_path: Vec<u32> = self.xpub_path.clone();
        if self.first_call {
            paths.push(initial_path);
            self.first_call = false;
        }
        for _ in 0..(n - paths.len()) {
            let mut last_index = self.xpub_path.len() - 1;
            if self.xpub_path[last_index] < self.max_depth {
                self.xpub_path[last_index] += 1;
            } else {
                self.xpub_path.truncate(last_index - 1);
                last_index = self.xpub_path.len() - 1;

                if self.xpub_path[last_index] < self.max_non_hardening_index {
                    self.xpub_path[last_index] += 1;
                    self.xpub_path.extend_from_slice(&[0, 0]);
                } else {
                    self.xpub_path.extend_from_slice(&[0, 0, 0]);
                }
            }
            paths.push(self.xpub_path.clone());
        }
        paths
    }
}
