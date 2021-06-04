use sha2::{Digest, Sha256};

// keeping it simple
pub type ID = String;

pub struct Node {
    ip: String,
    port: u16,
    id: ID,
}

impl Node {
    pub fn new(ip: String, port: u16) -> Self {
        let addr = format!("{}:{}", ip, port);

        // hashing addr
        let mut hasher = Sha256::new();
        hasher.update(addr.into_bytes());
        let result = hasher.finalize();

        let id = format!("{:X}", result);

        Node { ip, port, id }
    }
    pub fn get_info(&self) -> String {
        format!("{}:{}:{}", self.ip, self.port, self.id)
    }
}
