pub struct XpubPathWalker {
    xpub_path: Vec<u32>,
    max_depth: u32,
    first_call: bool,
}

const NON_HARDENING_MAX_INDEX: u32 = 0x7FFFFFFF;

impl Iterator for XpubPathWalker {
    type Item = Vec<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_path())
    }
}

impl XpubPathWalker {
    pub fn new(initial_path: Vec<u32>, max_depth: u32) -> Self {
        let xpub_path = [initial_path, vec![0, 0, 0]].concat();
        Self {
            xpub_path,
            max_depth,
            first_call: true,
        }
    }

    fn next_path(&mut self) -> Vec<u32> {
        if self.first_call {
            self.first_call = false;
            return self.xpub_path.clone();
        }

        let mut last_index = self.xpub_path.len() - 1;
        if self.xpub_path[last_index] < self.max_depth {
            self.xpub_path[last_index] += 1;
            return self.xpub_path.clone();
        }

        self.xpub_path.truncate(last_index - 1);
        last_index = self.xpub_path.len() - 1;

        if self.xpub_path[last_index] < NON_HARDENING_MAX_INDEX {
            self.xpub_path[last_index] += 1;
            self.xpub_path.extend_from_slice(&[0, 0]);
        } else {
            self.xpub_path.extend_from_slice(&[0, 0, 0]);
        }
        self.xpub_path.clone()
    }
}
