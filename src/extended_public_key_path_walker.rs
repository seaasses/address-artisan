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
}

impl Iterator for ExtendedPublicKeyPathWalker {
    type Item = Vec<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first_call {
            self.first_call = false;
            return Some(self.xpub_path.clone());
        }
        
        let mut last_index = self.xpub_path.len() - 1;
        if self.xpub_path[last_index] < self.max_depth {
            self.xpub_path[last_index] += 1;
        } else {
            if self.xpub_path[last_index - 2] < self.max_non_hardening_index {
                // [x, y, non_max_hardening, 0, max_index] -> [x, y, non_max_hardening+1, 0, 0]
                self.xpub_path[last_index - 2] += 1;
                self.xpub_path[last_index] = 0;
            } else {
                last_index += 1;
                // [x, y, max_hardening, 0, max_index] -> [x, y, max_hardening, 0, 0, 0 (new)]
                self.xpub_path.push(0);
                self.xpub_path[last_index - 1] = 0;
            }
        }
        Some(self.xpub_path.clone())
    }
}
