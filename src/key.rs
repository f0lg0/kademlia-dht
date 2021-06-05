use super::KEY_LEN;
use sha2::{Digest, Sha256};
use std::fmt::{Debug, Error, Formatter};

pub struct Key([u8; KEY_LEN]);

impl Key {
    pub fn new(input: String) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());

        // we know that the hash output is going to be 256 bits = 32 bytes
        let result = hasher.finalize();
        let mut hash = [0; KEY_LEN];

        for i in 0..result.len() {
            hash[i] = result[i];
        }

        Self(hash)
    }

    pub fn borrow(&self) -> &[u8; KEY_LEN] {
        &self.0
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for x in &self.0 {
            write!(f, "{:X}", x).expect("Failed to format contents of Key");
        }
        Ok(())
    }
}
