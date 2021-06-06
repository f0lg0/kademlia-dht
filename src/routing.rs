use super::key::Distance;
use super::node::Node;
use super::K_PARAM;
use super::N_BUCKETS;

#[derive(Debug)]
pub struct NodeAndDistance(pub Node, pub Distance);

#[derive(Debug)]
pub struct FindValueResult(Option<Node>, Option<String>);

#[derive(Debug)]
pub struct KBucket {
    nodes: Vec<Node>,
    size: usize,
}

#[derive(Debug)]
pub struct RoutingTable<'a> {
    node: &'a Node,
    kbuckets: Vec<KBucket>,
}

impl PartialEq for NodeAndDistance {
    fn eq(&self, other: &NodeAndDistance) -> bool {
        let mut equal = true;
        let mut i = 0;
        while equal && i < 32 {
            if self.1 .0[i] != other.1 .0[i] {
                equal = false;
            }

            i += 1;
        }

        equal
    }
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
