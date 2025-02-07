pub struct ExtendedPublicKeyPathWalker {
    xpub_path: Vec<u32>,
    max_index: u32,
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
            max_index: max_depth - 1,
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
        if self.xpub_path[last_index] < self.max_index {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easy_walk() {
        let initial_path = vec![123];
        let walker = ExtendedPublicKeyPathWalker::new(initial_path, 3);
        let expected_order = vec![vec![123, 0, 0, 0], vec![123, 0, 0, 1], vec![123, 0, 0, 2]];

        // use as iterator, as we want in the code
        for (index, path) in walker.enumerate() {
            assert_eq!(path, expected_order[index]);
            if index == expected_order.len() - 1 {
                break;
            }
        }
    }

    #[test]
    fn test_post_max_index() {
        let initial_path = vec![123];
        let walker = ExtendedPublicKeyPathWalker::new(initial_path, 3);
        let expected_order = vec![
            vec![123, 0, 0, 0],
            vec![123, 0, 0, 1],
            vec![123, 0, 0, 2],
            vec![123, 1, 0, 0],
            vec![123, 1, 0, 1],
            vec![123, 1, 0, 2],
            vec![123, 2, 0, 0],
            vec![123, 2, 0, 1],
            vec![123, 2, 0, 2],
        ];

        // use as iterator, as we want in the code
        for (index, path) in walker.enumerate() {
            assert_eq!(path, expected_order[index]);
            if index == expected_order.len() - 1 {
                break;
            }
        }
    }

    #[test]
    fn test_post_max_non_hardening_index() {
        let initial_path = vec![123];
        let mut walker = ExtendedPublicKeyPathWalker::new(initial_path, 3);
        walker.xpub_path[1] = 0x7FFFFFFE;
        let expected_order = vec![
            vec![123, 0x7FFFFFFE, 0, 0],
            vec![123, 0x7FFFFFFE, 0, 1],
            vec![123, 0x7FFFFFFE, 0, 2],
            vec![123, 0x7FFFFFFF, 0, 0],
            vec![123, 0x7FFFFFFF, 0, 1],
            vec![123, 0x7FFFFFFF, 0, 2],
            vec![123, 0x7FFFFFFF, 0, 0, 0],
            vec![123, 0x7FFFFFFF, 0, 0, 1],
            vec![123, 0x7FFFFFFF, 0, 0, 2],
            vec![123, 0x7FFFFFFF, 1, 0, 0],
            vec![123, 0x7FFFFFFF, 1, 0, 1],
            vec![123, 0x7FFFFFFF, 1, 0, 2],
            vec![123, 0x7FFFFFFF, 2, 0, 0],
        ];

        // use as iterator, as we want in the code
        for (index, path) in walker.enumerate() {
            assert_eq!(path, expected_order[index]);
            if index == expected_order.len() - 1 {
                break;
            }
        }
    }
}
