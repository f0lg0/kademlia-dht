use super::key::{Distance, Key};
use super::node::Node;
use super::K_PARAM;
use super::N_BUCKETS;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeAndDistance(pub Node, pub Distance);

#[derive(Debug, Serialize, Deserialize)]
pub struct FindValueResult(Option<Vec<NodeAndDistance>>, Option<String>);

#[derive(Debug)]
pub struct KBucket {
    pub nodes: Vec<Node>,
    pub size: usize,
}

#[derive(Debug)]
pub struct RoutingTable {
    pub node: Node,
    pub kbuckets: Vec<KBucket>,
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

// A k-bucket with index i stores contacts whose ids
// have a distance between 2^i and 2^i+1 to the own id
impl KBucket {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            size: K_PARAM,
        }
    }
}

impl RoutingTable {
    pub fn new(node: Node, bootstrap: Option<Node>) -> Self {
        let mut kbuckets: Vec<KBucket> = Vec::new();
        for _ in 0..N_BUCKETS {
            kbuckets.push(KBucket::new());
        }

        let mut ret = Self {
            node: node.clone(),
            kbuckets,
        };

        if let Some(bootstrap) = bootstrap {
            ret.update(bootstrap);
        }

        ret
    }

    pub fn update(&mut self, node: Node) {
        /*
            TODO: Adding a node:
                If the corresponding k-bucket stores less than k contacts
                and the new node is not already contained, the new node is added at the tail of the list.
                If the k-bucket contains the contact already, it is moved to the tail of the list.
                Should the appropriate k-bucket be full, then the contact at the head of the list is pinged.
                If it replies, then it is moved to the tail of the list and the new contact is not added.
                If it does not, the old contact is discarded and the new contact is added at the tail.
        */

        let bucket_idx = self.get_lookup_bucket_index(&node.id);
        let bucket = &mut self.kbuckets[bucket_idx];
        let node_idx = bucket.nodes.iter().position(|x| x.id == node.id);

        match node_idx {
            Some(i) => {
                println!(
                    "[VERBOSE] Routing::update --> Found exact index for node: {}, removing and inserting...",
                    i
                );
            }
            None => {
                println!("[VERBOSE] Routing::update --> Exact index for node has not been found, we can push no prob");
            }
        }
    }

    fn get_lookup_bucket_index(&self, key: &Key) -> usize {
        // https://stackoverflow.com/questions/2656642/easiest-way-to-find-the-correct-kademlia-bucket

        // given a bucket j, we are guaranteed that
        //  2^j <= distance(node, contact) < 2^(j+1)
        // a node with distance d will be put in the k-bucket with index i=⌊logd⌋

        let d = Distance::new(&self.node.id, key);
        for i in 0..super::KEY_LEN {
            for j in (0..8).rev() {
                if (d.0[i] >> (7 - j)) & 0x1 != 0 {
                    return i * 8 + j;
                }
            }
        }

        super::KEY_LEN * 8 - 1
    }
}
