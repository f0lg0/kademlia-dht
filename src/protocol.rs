use super::network;
use super::node::Node;
use super::routing;
use super::utils;

use crossbeam_channel;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Protocol {
    pub routes: Arc<Mutex<routing::RoutingTable>>,
    pub store: Arc<Mutex<HashMap<String, String>>>,
    pub rpc: Arc<network::Rpc>,
    pub node: Node,
}

impl Protocol {
    pub fn new(ip: String, port: u16, bootstrap: Option<Node>) -> Self {
        let node = Node::new(ip, port);

        let (rt_channel_sender, rt_channel_receiver) = crossbeam_channel::unbounded();

        let routes = routing::RoutingTable::new(
            node.clone(),
            bootstrap,
            rt_channel_sender.clone(),
            rt_channel_receiver.clone(),
        );

        let (rpc_channel_sender, rpc_channel_receiver) = mpsc::channel();

        let rpc = network::Rpc::new(node.clone());
        network::Rpc::open(rpc.clone(), rpc_channel_sender);

        let protocol = Self {
            routes: Arc::new(Mutex::new(routes)),
            store: Arc::new(Mutex::new(HashMap::new())),
            rpc: Arc::new(rpc),
            node: node.clone(),
        };

        protocol.clone().requests_handler(rpc_channel_receiver);
        protocol
            .clone()
            .rt_forwarder(rt_channel_sender, rt_channel_receiver);

        // performing node lookup on ourselves
        protocol.nodes_lookup(&node.id);

        protocol
    }

    fn rt_forwarder(
        self,
        sender: crossbeam_channel::Sender<utils::ChannelPayload>,
        receiver: crossbeam_channel::Receiver<utils::ChannelPayload>,
    ) {
        std::thread::spawn(move || {
            for req in receiver.iter() {
                let protocol = self.clone();
                let sender_clone = sender.clone();

                std::thread::spawn(move || match req {
                    utils::ChannelPayload::Request(payload) => match payload.0 {
                        network::Request::Ping => {
                            let success = protocol.ping(payload.1);
                            if success {
                                if let Err(_) = sender_clone
                                    .send(utils::ChannelPayload::Response(network::Response::Ping))
                                {
                                    eprintln!("[FAILED] Protocol::rt_forwared --> Receiver is dead, closing channel");
                                }
                            } else {
                                if let Err(_) = sender_clone.send(utils::ChannelPayload::NoData) {
                                    eprintln!("[FAILED] Protocol::rt_forwared --> Receiver is dead, closing channel");
                                }
                            }
                        }
                        _ => {
                            unimplemented!();
                        }
                    },
                    utils::ChannelPayload::Response(_) => {
                        eprintln!("[FAILED] Protocol::rt_forwarder --> Received a Response instead of a Request")
                    }
                    utils::ChannelPayload::NoData => {
                        eprintln!("[FAILED] Protocol::rt_forwarder --> Received a NoData instead of a Request")
                    }
                });
            }
        });
    }

    fn requests_handler(self, receiver: mpsc::Receiver<network::ReqWrapper>) {
        std::thread::spawn(move || {
            for req in receiver.iter() {
                let protocol = self.clone();

                std::thread::spawn(move || {
                    let res = protocol.craft_res(req);
                    protocol.reply(res);
                });
            }
        });
    }

    fn craft_res(&self, req: network::ReqWrapper) -> (network::Response, network::ReqWrapper) {
        let mut routes = self
            .routes
            .lock()
            .expect("[FAILED] Protocol::craft_res --> Failed to acquire mutex on Routes");

        // must craft node object because ReqWrapper contains only the src string addr
        let split = req.src.split(":");
        let parsed: Vec<&str> = split.collect();

        let src_node = Node::new(
            parsed[0].to_string(),
            parsed[1]
                .parse::<u16>()
                .expect("[FAILED] Protocol::craft_res --> Failed to parse Node port from address"),
        );
        routes.update(src_node);
        drop(routes);

        match req.payload {
            network::Request::Ping => (network::Response::Ping, req),
            network::Request::Store(ref k, ref v) => {
                // ref is used to borrow k and v, which are the contents of req

                let mut store = self
                    .store
                    .lock()
                    .expect("[FAILED] Protocol::craft_res --> Failed to acquire mutex on Store");
                store.insert(k.to_string(), v.to_string());

                (network::Response::Ping, req)
            }
            network::Request::FindNode(ref id) => {
                let routes = self
                    .routes
                    .lock()
                    .expect("[FAILED] Protocol::craft_res --> Failed to acquire mutex on Routes");

                let result = routes.get_closest_nodes(id, super::K_PARAM);

                (network::Response::FindNode(result), req)
            }
            network::Request::FindValue(ref k) => {
                let key = super::key::Key::new(k.to_string());
                let store = self
                    .store
                    .lock()
                    .expect("[FAILED] Protocol::craft_res --> Failed to acquire mutex on Store");

                let val = store.get(k);

                match val {
                    Some(v) => (
                        network::Response::FindValue(routing::FindValueResult::Value(
                            v.to_string(),
                        )),
                        req,
                    ),
                    None => {
                        let routes = self.routes.lock().expect(
                            "[FAILED] Protocol::craft_res --> Failed to acquire mutex on Routes",
                        );
                        (
                            network::Response::FindValue(routing::FindValueResult::Nodes(
                                routes.get_closest_nodes(&key, super::K_PARAM),
                            )),
                            req,
                        )
                    }
                }
            }
        }
    }

    fn reply(&self, packet_details: (network::Response, network::ReqWrapper)) {
        let msg = network::RpcMessage {
            token: packet_details.1.token,
            src: self.node.get_addr(),
            dst: packet_details.1.src,
            msg: network::Message::Response(packet_details.0),
        };

        self.rpc.send_msg(&msg);
    }

    pub fn ping(&self, dst: Node) -> bool {
        let res = utils::make_req_get_res(&self.rpc, network::Request::Ping, dst.clone());

        let mut routes = self
            .routes
            .lock()
            .expect("[FAILED] Protocol::ping --> Failed to acquire lock on Routes");

        if let Some(network::Response::Ping) = res {
            routes.update(dst);
            true
        } else {
            eprintln!(
                "[WARNING] Protocol::Ping --> No response, removing contact from routing table"
            );
            routes.remove(&dst);
            false
        }
    }

    pub fn store(&self, dst: Node, key: String, val: String) -> bool {
        let res =
            utils::make_req_get_res(&self.rpc, network::Request::Store(key, val), dst.clone());

        // since we get a ping, update our routing table
        let mut routes = self
            .routes
            .lock()
            .expect("[FAILED] Protocol::store --> Failed to acquire mutex on Routes");
        if let Some(network::Response::Ping) = res {
            routes.update(dst);
            true
        } else {
            routes.remove(&dst);
            false
        }
    }

    pub fn find_node(
        &self,
        dst: Node,
        id: super::key::Key,
    ) -> Option<Vec<routing::NodeAndDistance>> {
        let res = utils::make_req_get_res(&self.rpc, network::Request::FindNode(id), dst.clone());

        let mut routes = self
            .routes
            .lock()
            .expect("[FAILED] Protocol::find_node --> Failed to acquire mutex on Routes");
        if let Some(network::Response::FindNode(entries)) = res {
            routes.update(dst);
            Some(entries)
        } else {
            routes.remove(&dst);
            None
        }
    }

    pub fn find_value(&self, dst: Node, k: String) -> Option<routing::FindValueResult> {
        let res = utils::make_req_get_res(&self.rpc, network::Request::FindValue(k), dst.clone());

        let mut routes = self
            .routes
            .lock()
            .expect("[FAILED] Protocol::find_value --> Failed to acquire mutex on Routes");

        if let Some(network::Response::FindValue(val)) = res {
            routes.update(dst);
            Some(val)
        } else {
            routes.remove(&dst);
            None
        }
    }

    pub fn nodes_lookup(&self, id: &super::key::Key) -> Vec<routing::NodeAndDistance> {
        let mut ret: Vec<routing::NodeAndDistance> = Vec::new();

        let mut queried = HashSet::new();
        let routes = self
            .routes
            .lock()
            .expect("[FAILED] Protocol::nodes_lookup --> Failed to acquire mutex on Routes");

        let mut to_query = BinaryHeap::from(routes.get_closest_nodes(id, super::K_PARAM));
        drop(routes);

        for entry in &to_query {
            queried.insert(entry.clone());
        }

        while !to_query.is_empty() {
            let mut joins: Vec<std::thread::JoinHandle<Option<Vec<routing::NodeAndDistance>>>> =
                Vec::new();
            let mut queries: Vec<routing::NodeAndDistance> = Vec::new();
            let mut results: Vec<Option<Vec<routing::NodeAndDistance>>> = Vec::new();

            for _ in 0..super::ALPHA {
                match to_query.pop() {
                    Some(entry) => {
                        queries.push(entry);
                    }
                    None => {
                        break;
                    }
                }
            }

            for &routing::NodeAndDistance(ref node, _) in &queries {
                let n = node.clone();
                let id_clone = id.clone();
                let protocol_clone = self.clone();

                joins.push(std::thread::spawn(move || {
                    protocol_clone.find_node(n, id_clone)
                }));
            }

            for j in joins {
                results.push(j.join().expect(
                    "[FAILED] Protocol::nodes_lookup --> Failed to join thread while visiting nodes",
                ));
            }

            for (result, query) in results.into_iter().zip(queries) {
                if let Some(entries) = result {
                    ret.push(query);

                    for entry in entries {
                        if queried.insert(entry.clone()) {
                            to_query.push(entry);
                        }
                    }
                }
            }
        }

        ret.sort_by(|a, b| a.1.cmp(&b.1));
        ret.truncate(super::K_PARAM);

        ret
    }

    pub fn value_lookup(&self, k: String) -> (Option<String>, Vec<routing::NodeAndDistance>) {
        // NOTE: k and key are two different things, one is a string used to search for the corresponding value while the other is a key::Key

        let mut ret: Vec<routing::NodeAndDistance> = Vec::new();
        let key = super::key::Key::new(k.clone());
        let mut queried = HashSet::new();

        let routes = self
            .routes
            .lock()
            .expect("[FAILED] Protocol::value_lookup --> Failed to acquire mutex on Routes");
        let mut to_query = BinaryHeap::from(routes.get_closest_nodes(&key, super::K_PARAM));
        drop(routes);

        for entry in &to_query {
            queried.insert(entry.clone());
        }

        while !to_query.is_empty() {
            let mut joins: Vec<std::thread::JoinHandle<Option<routing::FindValueResult>>> =
                Vec::new();
            let mut queries: Vec<routing::NodeAndDistance> = Vec::new();
            let mut results: Vec<Option<routing::FindValueResult>> = Vec::new();

            for _ in 0..super::ALPHA {
                match to_query.pop() {
                    Some(entry) => {
                        queries.push(entry);
                    }
                    None => {
                        break;
                    }
                }
            }

            for &routing::NodeAndDistance(ref n, _) in &queries {
                let k_clone = k.clone();
                let node = n.clone();
                let protocol = self.clone();

                joins.push(std::thread::spawn(move || {
                    protocol.find_value(node, k_clone)
                }));
            }

            for j in joins {
                results.push(j.join().expect("[FAILED] Protocol::value_lookup --> Failed to join thread while searching for value"));
            }

            for (result, query) in results.into_iter().zip(queries) {
                if let Some(find_value_result) = result {
                    match find_value_result {
                        routing::FindValueResult::Nodes(entries) => {
                            // we didn't get the value we looked for
                            ret.push(query);
                            for entry in entries {
                                if queried.insert(entry.clone()) {
                                    to_query.push(entry);
                                }
                            }
                        }

                        routing::FindValueResult::Value(val) => {
                            ret.sort_by(|a, b| a.1.cmp(&b.1));
                            ret.truncate(super::K_PARAM);

                            return (Some(val), ret);
                        }
                    }
                }
            }
        }
        ret.sort_by(|a, b| a.1.cmp(&b.1));
        ret.truncate(super::K_PARAM);
        (None, ret)
    }

    pub fn put(&self, k: String, v: String) {
        let candidates = self.nodes_lookup(&super::key::Key::new(k.clone()));

        for routing::NodeAndDistance(node, _) in candidates {
            let protocol_clone = self.clone();
            let k_clone = k.clone();
            let v_clone = v.clone();

            std::thread::spawn(move || {
                protocol_clone.store(node, k_clone, v_clone);
            });
        }
    }

    pub fn get(&self, k: String) -> Option<String> {
        let (val, mut nodes) = self.value_lookup(k.clone());

        val.map(|v| {
            if let Some(routing::NodeAndDistance(target, _)) = nodes.pop() {
                self.store(target, k, v.clone());
            } else {
                self.store(self.node.clone(), k, v.clone());
            }

            v
        })
    }
}
