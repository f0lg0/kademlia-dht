pub mod node;
pub mod utils; // TODO: remove pub (left it to avoid warnings)

#[cfg(test)]
mod tests {
    use super::node::Node;
    use super::utils;

    #[test]
    fn kademlia_node() {
        let node = Node::new(utils::get_local_ip().unwrap(), 1337);
        let node_info = node.get_info();

        assert_eq!(
            node_info,
            "192.168.1.10:1337:B1F14199A00EA18325688FEE9DCD3E48E9269276892C2F3E66135EA15C5C90EB"
        )
    }
}
