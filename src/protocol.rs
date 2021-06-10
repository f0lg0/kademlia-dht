use super::key;
use super::network;
use super::node::Node;
use super::routing::RoutingTable;

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Protocol {
    pub routes: Arc<Mutex<RoutingTable>>,
    pub store: Arc<Mutex<HashMap<String, String>>>,
    pub rpc: Arc<network::Rpc>,
    pub node: Node,
}

impl Protocol {
    // TODO: missing bootstrap node
    pub fn new(ip: String, port: u16) -> Self {
        let node = Node::new(ip, port);
        let routes = RoutingTable::new(node.clone());

        let (sender, receiver) = mpsc::channel();
        let rpc = network::Rpc::new(node.clone());
        network::Rpc::open(rpc.clone(), sender);

        let node = Self {
            routes: Arc::new(Mutex::new(routes)),
            store: Arc::new(Mutex::new(HashMap::new())),
            rpc: Arc::new(rpc),
            node,
        };

        node.clone().requests_handler(receiver);

        node
    }

    fn requests_handler(self, receiver: mpsc::Receiver<network::Request>) {
        // TODO: return response

        std::thread::spawn(move || {
            println!(
                "*** {} Requests Handler got: {:?}",
                self.node.get_info(),
                receiver
                    .recv()
                    .expect("request_handler --> Errors while receiving from channel sender")
            );
        });
    }

    pub fn ping(&self, dst: Node) {
        let msg = network::RpcMessage {
            token: key::Key::new(String::from("Hello node1")),
            src: self.node.get_addr(),
            dst: dst.get_addr(),
            msg: network::Message::Request(network::Request::Ping),
        };
        self.rpc.send_msg(&msg, &dst.get_addr());
        println!("[+] Protocol::ping --> Ping message sent!")
    }
}
