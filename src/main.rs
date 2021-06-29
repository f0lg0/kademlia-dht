extern crate kademlia_dht;
use kademlia_dht::node::Node;
use kademlia_dht::protocol::Protocol;
use kademlia_dht::utils;

const BIG_TEST: bool = true;

// be careful with the net size, for example my computer can't spawn too many threads
// messages may also exceed the buffer size used for streaming (see issue #1)
const NET_SIZE: usize = 10;

fn test_big_net() {
	let mut interfaces: Vec<Protocol> = Vec::with_capacity(NET_SIZE);
	let mut base_port = 8000;

	let root = Node::new(utils::get_local_ip().unwrap(), 7999);
	let root_interface = Protocol::new(root.ip.clone(), root.port.clone(), None);
	root_interface.put("MAIN_KEY".to_owned(), "MAIN_VALUE".to_owned());

	for i in 0..(NET_SIZE - 1) {
		let node = Node::new(utils::get_local_ip().unwrap(), base_port);

		interfaces.push(Protocol::new(node.ip, node.port, Some(root.clone())));
		println!(
			"[+] Created interface for index: {} on port: {}",
			i, base_port
		);

		base_port += 1;
		// thread::sleep(time::Duration::from_secs(1));
	}

	for (index, interface) in interfaces.iter().enumerate() {
		println!("[+] Putting <key, value> pair for index: {}", index);
		interface.put(format!("key_{}", index), format!("value_{}", index));
		// thread::sleep(time::Duration::from_secs(1));
	}

	for (index, interface) in interfaces.iter().enumerate() {
		let res = interface.get(format!("key_{}", index));
		println!("[*] Looking for key_{}, got {}", index, res.unwrap());
		// thread::sleep(time::Duration::from_secs(1));
	}
}

fn main() {
	if BIG_TEST {
		test_big_net();
	} else {
		let node0 = Node::new(utils::get_local_ip().unwrap(), 1337);
		println!("[+] Created node0: {:?}", node0);

		let node1 = Node::new(utils::get_local_ip().unwrap(), 1338);
		println!("[+] Created node1: {:?}", node1);

		let node2 = Node::new(utils::get_local_ip().unwrap(), 1339);
		println!("[+] Created node2: {:?}", node2);

		let interface0 = Protocol::new(node0.ip.clone(), node0.port.clone(), None);
		println!("[+] Initialized Kademlia Protocol for node0 (interface0)");

		let interface1 = Protocol::new(node1.ip.clone(), node1.port.clone(), Some(node0.clone()));
		println!("[+] Initialized Kademlia Protocol for node1 (interface1)");

		let interface2 = Protocol::new(node2.ip.clone(), node2.port.clone(), Some(node0.clone()));
		println!("[+] Initialized Kademlia Protocol for node2 (interface2)");

		println!("\n--------------------------------------");
		println!("Calling Kademlia API");

		interface0.put("some_key".to_owned(), "some_value".to_owned());
		println!("\t[*] node0 > called PUT for key: 'some_key' and value: 'some_value'");

		let get_res = interface2.get("some_key".to_owned());
		println!("\t[*] node2 > called GET on key: 'some_key'");
		println!("\t\t[+] Extracted: {:?}", get_res);
		println!("--------------------------------------\n");

		utils::dump_interface_state(&interface0, "dumps/interface0.json");
		utils::dump_interface_state(&interface1, "dumps/interface1.json");
		utils::dump_interface_state(&interface2, "dumps/interface2.json");
		println!("[*] Dumped protocol states for node0, node1 and node2. Check out the 'dumps' folder for a complete tracelog");
		println!("Exiting...");
	}
}
