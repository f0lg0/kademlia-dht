use super::key::Key;
use super::node::*;
use super::routing::FindValueResult;
use super::routing::NodeAndDistance;

enum Request {
    Ping,
    Store(String, String),
    FindNode(Key),
    FindValue(String),
}

enum Response {
    Ping,
    FindNode(Vec<NodeAndDistance>),
    FindValue(FindValueResult),
}

enum Message {
    Abort,
    Request(Request),
    Response(Response),
}

struct RpcMessage {
    token: Key,
    src: Node,
    dst: Node,
    msg: Message,
}

pub fn ping() {}
pub fn store() {}
pub fn find_node() {}
pub fn find_value() {}
