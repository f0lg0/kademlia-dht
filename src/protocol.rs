use super::network::Rpc;
use super::node::Node;
use super::routing::RoutingTable;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Protocol {
    routes: Arc<Mutex<RoutingTable>>,
    store: Arc<Mutex<HashMap<String, String>>>,
    rpc: Arc<Rpc>,
    node: Node,
}

impl Protocol {
    // TODO: missing bootstrap node
    pub fn new(ip: String, port: u16) -> Self {
        let node = Node::new(ip, port);
        let routes = RoutingTable::new(node.clone());
        let rpc = Rpc::new(node.clone());

        Self {
            routes: Arc::new(Mutex::new(routes)),
            store: Arc::new(Mutex::new(HashMap::new())),
            rpc: Arc::new(rpc),
            node,
        }
    }
}
