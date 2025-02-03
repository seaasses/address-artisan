use crate::xpub::ExtendedPublicKeyDeriver;

pub struct XpubPubkeyHashWalker {
    xpub_path: Vec<u32>,
    max_depth: u32,
    first_call: bool,
    xpub_public_key_deriver: ExtendedPublicKeyDeriver,
}

impl Iterator for XpubPubkeyHashWalker {
    type Item = [u8; 20];

    fn next(&mut self) -> Option<Self::Item> {
        self.next_pubkey_hash()
    }
}

impl XpubPubkeyHashWalker {
    pub fn new(xpub: String, initial_path: Vec<u32>, max_depth: u32) -> Self {
        let xpub_path = [initial_path, vec![0, 0, 0]].concat();
        let xpub_public_key_deriver = ExtendedPublicKeyDeriver::new(&xpub);
        Self {
            xpub_path,
            max_depth,
            first_call: true,
            xpub_public_key_deriver,
        }
    }

    fn next_pubkey_hash(&mut self) -> Option<[u8; 20]> {
        self.next_path();
        let pubkey_hash = self
            .xpub_public_key_deriver
            .get_pubkey_hash_160(&self.xpub_path);
        if pubkey_hash.is_err() {
            return None;
        }
        Some(pubkey_hash.unwrap())
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

            if self.xpub_path[last_index] < self.xpub_public_key_deriver.non_hardening_max_index {
                self.xpub_path[last_index] += 1;
                self.xpub_path.extend_from_slice(&[0, 0]);
            } else {
                self.xpub_path.extend_from_slice(&[0, 0, 0]);
            }
        }
    }
}
