extern crate kademlia_dht;
use kademlia_dht::protocol::Protocol;
use kademlia_dht::utils;

fn main() {
	let interface0 = Protocol::new(utils::get_local_ip().unwrap(), 1337, None);
	let interface1 = Protocol::new(utils::get_local_ip().unwrap(), 1338, None);

	interface0.ping(interface1.node.clone());
	interface1.ping(interface0.node.clone());
}
