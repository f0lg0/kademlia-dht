pub mod key;
pub mod network;
pub mod node;
pub mod protocol;
pub mod routing;
pub mod utils;

// 256 bits --> 32 bytes
const KEY_LEN: usize = 32;

// a list for each bit of the node ID
// 32*8 --> 256
const N_BUCKETS: usize = KEY_LEN * 8;

// number entries in a list
const K_PARAM: usize = 20;

// buffer size used for streaming UDP
const BUF_SIZE: usize = 4096;

// response timeout 5000ms
const TIMEOUT: u64 = 5000;

// number of concurrent lookups in node lookup
const ALPHA: usize = 3;

const VERBOSE: bool = false;

#[cfg(test)]
mod tests {
    use super::key::Distance;
    use super::key::Key;
    use super::node::Node;
    use super::protocol::Protocol;
    use super::routing::NodeAndDistance;
    use super::utils;

    use std::{thread, time};

    #[test]
    fn create_key() {
        let input_str = String::from("test_string");
        let k = Key::new(input_str);
        println!("test_string: {:?}", k);
    }
    #[test]
    fn create_node() {
        let node = Node::new(utils::get_local_ip().unwrap(), 1337);
        let node_info = node.get_info();

        println!("node: {}", node_info);

        assert_eq!(
            node_info,
            "192.168.1.10:1337:B1F14199A00EA18325688FEE9DCD3E48E9269276892C2F3E66135EA15C5C90EB"
        )
    }

    #[test]
    fn distance_between_nodes() {
        let node0 = Node::new(utils::get_local_ip().unwrap(), 1337);
        let node1 = Node::new(utils::get_local_ip().unwrap(), 1338);

        let dist = Distance::new(&node0.id, &node1.id);
        println!(
            "node0.id: {:?}, node1.id: {:?}, dist: {:?}",
            node0.id, node1.id, dist
        );
    }

    #[test]
    fn compare_distance() {
        let node0 = Node::new(utils::get_local_ip().unwrap(), 1337);
        let node1 = Node::new(utils::get_local_ip().unwrap(), 1338);

        let dist = Distance::new(&node0.id, &node1.id);
        let nd0 = NodeAndDistance(node0.clone(), dist.clone());
        let nd1 = NodeAndDistance(node1.clone(), dist.clone());

        assert_eq!(nd0, nd1);
    }

    #[test]
    fn start_protocol() {
        let root_node = Node::new(utils::get_local_ip().unwrap(), 1339);

        let interface0 = Protocol::new(utils::get_local_ip().unwrap(), 1339, None);
        let interface1 = Protocol::new(utils::get_local_ip().unwrap(), 1340, Some(root_node));

        interface0.ping(interface1.node.clone());
        interface0.ping(interface1.node.clone());
        interface1.ping(interface0.node.clone());

        thread::sleep(time::Duration::from_secs(3));
        utils::dump_interface_state(&interface0, "dumps/interface0.json");
        utils::dump_interface_state(&interface1, "dumps/interface1.json");
    }

    #[test]
    fn dump_interface() {
        let interface = Protocol::new(utils::get_local_ip().unwrap(), 1401, None);
        utils::dump_interface_state(&interface, "dumps/interface.json");
        println!("[*] Dumped interface to file [*]");
    }
}
