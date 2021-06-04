use hex;
use sha2::{Digest, Sha256};
// keeping it simple
pub type ID = [u8; 32];

pub struct Node {
    ip: String,
    port: u16,
    id: ID,
}

impl Node {
    pub fn new(ip: String, port: u16) -> Self {
        let str_addr = format!("{}:{}", ip, port);
        let addr = str_addr.as_bytes();

        // hashing addr
        let mut hasher = Sha256::new();
        hasher.update(&addr);

        let result = hasher.finalize();
        let hash = format!("{:X}", result);
        let decoded = hex::decode(hash).expect("Decoding failed");

        let mut id = [0; 32];
        for i in 0..decoded.len() {
            id[i] = decoded[i];
        }

        Node { ip, port, id }
    }
    pub fn get_info(&self) -> String {
        let mut parsed_id = hex::encode(&self.id);
        parsed_id = parsed_id.to_ascii_uppercase();

        format!("{}:{}:{}", self.ip, self.port, parsed_id)
    }
}
