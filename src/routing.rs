use super::node::Node;

pub struct KBucket {
    nodes: Vec<Node>,
    size: u32,
}

pub struct RoutingTable<'a> {
    node: &'a Node,
    kbuckets: Vec<KBucket>,
}

impl KBucket {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            size: 20,
        }
    }
}

impl<'a> RoutingTable<'a> {
    fn new(node: &'a Node) -> Self {
        Self {
            node,
            kbuckets: Vec::new(),
        }
    }
}
