use std::{thread, time};

extern crate kademlia_dht;
use kademlia_dht::*;

fn main() {
    let node0 = protocol::create_node(utils::get_local_ip().unwrap(), 1337);
    let node1 = protocol::create_node(utils::get_local_ip().unwrap(), 1338);

    let rpc0 = network::Rpc::new(node0.clone());
    let rpc1 = network::Rpc::new(node1.clone());

    network::Rpc::open(rpc0.clone());
    network::Rpc::open(rpc1.clone());

    let msg = network::RpcMessage {
        token: key::Key::new(String::from("test")),
        src: node0.get_addr(),
        dst: node1.get_addr(),
        msg: network::Message::Request(network::Request::Ping),
    };

    let sec = time::Duration::from_secs(3);
    loop {
        thread::sleep(sec);
        rpc0.send_msg(&msg, &node1.get_addr());
    }
}
