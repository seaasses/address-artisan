pub struct XpubPathWalker {
    xpub_path: Vec<u32>,
    max_depth: u32,
    first_call: bool,
    max_non_hardening_index: u32,
}

impl Iterator for XpubPathWalker {
    type Item = Vec<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_path();
        return Some(self.xpub_path.clone());
    }
}

impl XpubPathWalker {
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

    fn next_path(&mut self) {
        if self.first_call {
            self.first_call = false;
            return;
        }

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
    }
}
