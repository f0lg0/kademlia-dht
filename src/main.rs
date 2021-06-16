extern crate kademlia_dht;
use kademlia_dht::node::Node;
use kademlia_dht::protocol::Protocol;
use kademlia_dht::utils;

fn main() {
	let root = Node::new(utils::get_local_ip().unwrap(), 1337);
	let interface0 = Protocol::new(root.ip.clone(), root.port.clone(), None);
	let interface1 = Protocol::new(utils::get_local_ip().unwrap(), 1338, Some(root));

	// utils::dump_interface_state(&interface0, "dumps/interface0.json");
	// utils::dump_interface_state(&interface1, "dumps/interface1.json");

	interface0.ping(interface1.node.clone());
	// interface1.ping(interface0.node.clone());

	utils::dump_interface_state(&interface0, "dumps/interface0.json");
	utils::dump_interface_state(&interface1, "dumps/interface1.json");
}
