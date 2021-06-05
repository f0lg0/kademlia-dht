use super::key::Key;

pub struct Node {
    ip: String,
    port: u16,
    id: Key,
}

impl Node {
    pub fn new(ip: String, port: u16) -> Self {
        let addr = format!("{}:{}", ip, port);
        let id = Key::new(addr);

        Node { ip, port, id }
    }
    pub fn get_info(&self) -> String {
        let mut parsed_id = hex::encode(self.id.borrow());
        parsed_id = parsed_id.to_ascii_uppercase();

        format!("{}:{}:{}", self.ip, self.port, parsed_id)
    }
}
