use super::KEY_LEN;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{Debug, Error, Formatter};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Key(pub [u8; KEY_LEN]);

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
}

impl Debug for Key {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for x in &self.0 {
            write!(f, "{:X}", x).expect("Failed to format contents of Key");
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Distance(pub [u8; KEY_LEN]);

impl Distance {
    pub fn new(k1: &Key, k2: &Key) -> Distance {
        let mut ret = [0; KEY_LEN];
        for i in 0..KEY_LEN {
            ret[i] = k1.0[i] ^ k2.0[i];
        }

        Self(ret)
    }

    // TODO: look better into this
    pub fn zeroes_in_prefix(&self) -> usize {
        for i in 0..KEY_LEN {
            for j in 8usize..0 {
                if (self.0[i] >> (7 - j)) & 0x1 != 0 {
                    return i * 8 + j;
                }
            }
        }
        KEY_LEN * 8 - 1
    }
}

impl Debug for Distance {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for x in &self.0 {
            write!(f, "{:X}", x).expect("Failed to format contents of Key");
        }
        Ok(())
    }
}
