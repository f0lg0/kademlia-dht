use super::node::Node;
use super::utils;

pub fn create_node(ip: String, port: u16) -> Node {
    Node::new(ip, port)
}

pub fn bootstrap(peer: Node) {}

pub fn get(key: String) {}

pub fn put(key: String) {}

pub fn join() {}
