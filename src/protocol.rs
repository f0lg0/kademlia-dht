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

        let protocol = Self {
            routes: Arc::new(Mutex::new(routes)),
            store: Arc::new(Mutex::new(HashMap::new())),
            rpc: Arc::new(rpc),
            node,
        };

        protocol.clone().requests_handler(receiver);

        // TODO: perform lookup on ourselves

        protocol
    }

    fn requests_handler(self, receiver: mpsc::Receiver<network::ReqWrapper>) {
        println!(
            "[*] Protocol::requests_handler --> Starting Requests Handler for receiver: {} [*]",
            &self.node.get_addr()
        );
        std::thread::spawn(move || {
            for req in receiver.iter() {
                let protocol = self.clone();

                println!(
                    "[VERBOSE] Protocol::requests_handler --> Spawning thread to handle {:?}",
                    &req
                );
                std::thread::spawn(move || {
                    let res = protocol.craft_res(req);
                    protocol.reply(res);
                });
            }
        });
    }

    fn craft_res(&self, req: network::ReqWrapper) -> (network::Response, String) {
        println!(
            "\t[VERBOSE] Protocol::requests_handler --> Parsing: {:?}",
            &req
        );

        let mut routes = self
            .routes
            .lock()
            .expect("Failed to acquire mutex on 'Routes' struct");

        // must craft node object because ReqWrapper contains only the src string addr
        let split = req.src.split(":");
        let parsed: Vec<&str> = split.collect();

        let src_node = Node::new(
            parsed[0].to_string(),
            parsed[1]
                .parse::<u16>()
                .expect("[FAILED] Failed to parse Node port from address"),
        );
        routes.update(src_node);
        drop(routes);

        match req.payload {
            network::Request::Ping => (network::Response::Ping, req.src),
            network::Request::Store(_, _) => (network::Response::Ping, req.src),
            network::Request::FindNode(_) => (network::Response::Ping, req.src),
            network::Request::FindValue(_) => (network::Response::Ping, req.src),
        }
    }

    fn reply(&self, res: (network::Response, String)) {
        println!("\t[VERBOSE] Replying with {:?} to {}", &res.0, &res.1);

        let msg = network::RpcMessage {
            token: key::Key::new(String::from("pong")),
            src: self.node.get_addr(),
            dst: res.1,
            msg: network::Message::Response(network::Response::Ping),
        };

        self.rpc.send_msg(&msg);
    }

    pub fn ping(&self, dst: Node) -> bool {
        println!("[STATUS] Protocol::ping --> Pinging...");
        let res = self
            .rpc
            .make_request(network::Request::Ping, dst.clone())
            .recv()
            .expect("Failed to receive data from channel while awaiting Ping response");

        let mut routes = self
            .routes
            .lock()
            .expect("Failed to acquire lock on routes");

        if let Some(network::Response::Ping) = res {
            println!("[STATUS] Protocol::Ping --> Got Pong");
            routes.update(dst);
            true
        } else {
            println!(
                "[FAILED] Protocol::Ping --> No response, removing contact from routing tablr"
            );
            routes.remove(&dst);
            false
        }
    }
}
