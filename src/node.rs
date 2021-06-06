use super::key::Key;

#[derive(Clone, Debug)]
pub struct Node {
    pub ip: String,
    pub port: u16,
    pub id: Key,
}

impl Node {
    pub fn new(ip: String, port: u16) -> Self {
        let addr = format!("{}:{}", ip, port);
        let id = Key::new(addr);

        Node { ip, port, id }
    }
    pub fn get_info(&self) -> String {
        let mut parsed_id = hex::encode(self.id.0);
        parsed_id = parsed_id.to_ascii_uppercase();

        format!("{}:{}:{}", self.ip, self.port, parsed_id)
    }
}
