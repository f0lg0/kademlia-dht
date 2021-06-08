use super::key::Key;
use super::node::*;
use super::routing::FindValueResult;
use super::routing::NodeAndDistance;

use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

pub enum Request {
    Ping,
    Store(String, String),
    FindNode(Key),
    FindValue(String),
}

pub enum Response {
    Ping,
    FindNode(Vec<NodeAndDistance>),
    FindValue(FindValueResult),
}

pub enum Message {
    Abort,
    Request(Request),
    Response(Response),
}

pub struct RpcMessage {
    token: Key,
    src: Node,
    dst: Node,
    msg: Message,
}

pub struct Rpc {
    socket: Arc<UdpSocket>,
    pending: Arc<Mutex<HashMap<Key, Sender<Option<Response>>>>>,
    node: Node,
}

impl Rpc {
    pub fn new(node: Node, dst: Node) -> Self {
        let socket = UdpSocket::bind(node.get_addr()).expect(format!(
            "Rpc::new --> Error while binding UdpSocket to addr {}",
            node.get_addr()
        ));

        Self {
            socket: Arc::new(socket),
            pending: Arc::new(Mutex::new(HashMap::new())),
            node,
        }
    }
    pub fn open(rpc: &Rpc) {
        // TODO: spawn thread and start listening
    }
}

// pub fn ping() {}
// pub fn store() {}
// pub fn find_node() {}
// pub fn find_value() {}
