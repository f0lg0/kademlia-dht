pub mod key;
pub mod network;
pub mod node;
pub mod protocol;
pub mod routing;
pub mod utils;

const N_BUCKETS: usize = 20;
const K_PARAM: usize = 8;
const KEY_LEN: usize = 32;
const BUF_SIZE: usize = 4096;

#[cfg(test)]
mod tests {
    use super::key::Distance;
    use super::key::Key;
    use super::network::*;
    use super::protocol::create_node;
    use super::routing::NodeAndDistance;
    use super::utils;

    #[test]
    fn create_key() {
        let input_str = String::from("test_string");
        let k = Key::new(input_str);
        println!("test_string: {:?}", k);
    }
    #[test]
    fn create_kademlia_node() {
        let node = create_node(utils::get_local_ip().unwrap(), 1337);
        let node_info = node.get_info();

        println!("node: {}", node_info);

        assert_eq!(
            node_info,
            "192.168.1.10:1337:B1F14199A00EA18325688FEE9DCD3E48E9269276892C2F3E66135EA15C5C90EB"
        )
    }

    #[test]
    fn distance_between_nodes() {
        let node0 = create_node(utils::get_local_ip().unwrap(), 1337);
        let node1 = create_node(utils::get_local_ip().unwrap(), 1338);

        let dist = Distance::new(&node0.id, &node1.id);
        println!(
            "node0.id: {:?}, node1.id: {:?}, dist: {:?}",
            node0.id, node1.id, dist
        );
    }

    #[test]
    fn compare_distance() {
        let node0 = create_node(utils::get_local_ip().unwrap(), 1337);
        let node1 = create_node(utils::get_local_ip().unwrap(), 1338);

        let dist = Distance::new(&node0.id, &node1.id);
        let nd0 = NodeAndDistance(node0.clone(), dist.clone());
        let nd1 = NodeAndDistance(node1.clone(), dist.clone());

        // assert_eq!(nd0, nd1);
        //      ^^^^^^^^^^^^^^^^^^^^^ `NodeAndDistance` cannot be formatted using `{:?}`
        let mut are_eq = false;
        if nd0 == nd1 {
            are_eq = true;
        }

        assert_eq!(are_eq, true);
    }

    #[test]
    fn open_rpc() {
        let node0 = create_node(utils::get_local_ip().unwrap(), 1337);
        let node1 = create_node(utils::get_local_ip().unwrap(), 1338);

        let rpc0 = Rpc::new(node0.clone(), node1.clone());
        let rpc1 = Rpc::new(node1.clone(), node0.clone());

        Rpc::open(rpc0.clone());
        Rpc::open(rpc1.clone());
    }
}
