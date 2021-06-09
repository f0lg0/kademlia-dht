use super::network::Rpc;
use super::node::Node;
use super::routing::RoutingTable;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Protocol {
    pub routes: Arc<Mutex<RoutingTable>>,
    pub store: Arc<Mutex<HashMap<String, String>>>,
    pub rpc: Arc<Rpc>,
    pub node: Node,
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
