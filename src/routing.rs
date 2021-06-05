use super::node::Node;
use super::K_PARAM;
use super::N_BUCKETS;

pub struct KBucket {
    nodes: Vec<Node>,
    size: usize,
}

pub struct RoutingTable<'a> {
    node: &'a Node,
    kbuckets: Vec<KBucket>,
}

impl KBucket {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            size: K_PARAM,
        }
    }
}

impl<'a> RoutingTable<'a> {
    fn new(node: &'a Node) -> Self {
        let mut kbuckets: Vec<KBucket> = Vec::new();
        for _ in 0..N_BUCKETS {
            kbuckets.push(KBucket::new());
        }

        Self { node, kbuckets }
    }
}
