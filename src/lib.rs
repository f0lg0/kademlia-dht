pub mod key;
pub mod network;
pub mod node;
pub mod protocol;
pub mod routing;
pub mod utils;

const N_BUCKETS: usize = 20;
const K_PARAM: usize = 8;
const KEY_LEN: usize = 32;

#[cfg(test)]
mod tests {
    use super::key::Key;
    use super::protocol::create_node;
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
}
