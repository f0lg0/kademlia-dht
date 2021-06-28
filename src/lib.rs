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
const BUF_SIZE: usize = 4096 * 2;

// response timeout 5000ms
const TIMEOUT: u64 = 5000;

// number of concurrent lookups in node lookup
const ALPHA: usize = 3;

const VERBOSE: bool = false;

#[cfg(test)]
mod tests {
    use super::key::Distance;
    use super::node::Node;
    use super::protocol::Protocol;
    use super::routing::NodeAndDistance;
    use super::utils;

    #[test]
    fn compare_distance() {
        let node0 = Node::new(utils::get_local_ip().unwrap(), 1335);
        let node1 = Node::new(utils::get_local_ip().unwrap(), 1336);

        let dist = Distance::new(&node0.id, &node1.id);
        let nd0 = NodeAndDistance(node0.clone(), dist.clone());
        let nd1 = NodeAndDistance(node1.clone(), dist.clone());

        assert_eq!(nd0, nd1);
    }

    #[test]
    fn main_test() {
        let node0 = Node::new(utils::get_local_ip().unwrap(), 1337);
        let node1 = Node::new(utils::get_local_ip().unwrap(), 1338);
        let node2 = Node::new(utils::get_local_ip().unwrap(), 1339);

        let interface0 = Protocol::new(node0.ip.clone(), node0.port.clone(), None);
        let interface1 = Protocol::new(node1.ip.clone(), node1.port.clone(), Some(node0.clone()));
        let interface2 = Protocol::new(node2.ip.clone(), node2.port.clone(), Some(node0.clone()));

        interface0.put("some_key".to_owned(), "some_value".to_owned());
        let get_res_1 = interface1.get("some_key".to_owned());
        let get_res_2 = interface2.get("some_key".to_owned());

        assert_eq!("some_value", get_res_1.clone().unwrap());
        assert_eq!(get_res_1.unwrap(), get_res_2.unwrap());
    }

    #[test]
    fn dump_interface() {
        let interface = Protocol::new(utils::get_local_ip().unwrap(), 1400, None);
        utils::dump_interface_state(&interface, "dumps/interface.json");
    }
}
