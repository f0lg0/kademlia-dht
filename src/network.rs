use super::key::Key;
use super::node::*;

enum Request {
    Ping,
    Store(String, String),
    FindNode(Key),
    FindValue(String),
}

// TODO: attach correct return data to fields
enum Response {
    Ping,
    FindNode,
    FindValue,
}

enum Message {
    Abort,
    Request(Request),
    Response(Response),
}

struct RpcMessage<'a> {
    token: Key,
    src: &'a Node,
    dst: &'a Node,
    msg: Message,
}

pub fn ping() {}
pub fn store() {}
pub fn find_node() {}
pub fn find_value() {}
