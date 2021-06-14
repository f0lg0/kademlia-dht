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
    pub fn new(ip: String, port: u16, bootstrap: Option<Node>) -> Self {
        let node = Node::new(ip, port);
        println!("[VERBOSE] Protocol::new --> Node created");

        let routes = RoutingTable::new(node.clone(), bootstrap);
        println!("[VERBOSE] Protocol::new --> Routes created");

        let (sender, receiver) = mpsc::channel();
        let rpc = network::Rpc::new(node.clone());
        network::Rpc::open(rpc.clone(), sender);
        println!("[VERBOSE] Protocol::new --> RPC created");

        let node = Self {
            routes: Arc::new(Mutex::new(routes)),
            store: Arc::new(Mutex::new(HashMap::new())),
            rpc: Arc::new(rpc),
            node,
        };

        node.clone().requests_handler(receiver);

        node
    }

    fn requests_handler(self, receiver: mpsc::Receiver<network::ReqWrapper>) {
        println!(
            "[*] Protocol::requests_handler --> Starting Requests Handler for receiver: {} [*]",
            self.node.get_addr()
        );
        std::thread::spawn(move || {
            for req in receiver.iter() {
                let node = self.clone();

                println!(
                    "[VERBOSE] Protocol::requests_handler --> Spawning thread to handle {:?}",
                    &req
                );
                std::thread::spawn(move || {
                    let res = node.craft_res(req);
                    node.reply(res);
                });
            }
        });
    }

    fn craft_res(&self, req: network::ReqWrapper) -> (network::Response, String) {
        println!(
            "\t[VERBOSE] Protocol::requests_handler --> Parsing: {:?}",
            req
        );

        match req.payload {
            network::Request::Ping => (network::Response::Ping, req.src),
            network::Request::Store(_, _) => (network::Response::Ping, req.src),
            network::Request::FindNode(_) => (network::Response::Ping, req.src),
            network::Request::FindValue(_) => (network::Response::Ping, req.src),
        }
    }

    fn reply(&self, res: (network::Response, String)) {
        println!("\t[VERBOSE] Replying with {:?} to {}", res.0, res.1);

        let msg = network::RpcMessage {
            token: key::Key::new(String::from("pong")),
            src: self.node.get_addr(),
            dst: res.1,
            msg: network::Message::Request(network::Request::Ping),
        };

        self.rpc.send_msg(&msg);
    }

    pub fn ping(&self, dst: Node) {
        let msg = network::RpcMessage {
            token: key::Key::new(String::from("ping")),
            src: self.node.get_addr(),
            dst: dst.get_addr(),
            msg: network::Message::Request(network::Request::Ping),
        };
        self.rpc.send_msg(&msg);
        println!(
            "[+] Protocol::ping --> Ping from {} to {} with token {:?} was sent!",
            msg.src, msg.dst, msg.token
        );
    }
}
