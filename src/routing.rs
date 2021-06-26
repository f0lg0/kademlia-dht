use super::key::{Distance, Key};
use super::network;
use super::node::Node;
use super::utils::ChannelPayload;
use super::K_PARAM;
use super::N_BUCKETS;

use crossbeam_channel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, Hash, Clone)]
pub struct NodeAndDistance(pub Node, pub Distance);

#[derive(Debug, Serialize, Deserialize)]
pub enum FindValueResult {
    Nodes(Vec<NodeAndDistance>),
    Value(String),
}

#[derive(Debug)]
pub struct KBucket {
    pub nodes: Vec<Node>,
    pub size: usize,
}

#[derive(Debug)]
pub struct RoutingTable {
    pub node: Node,
    pub kbuckets: Vec<KBucket>,
    pub sender: crossbeam_channel::Sender<ChannelPayload>,
    pub receiver: crossbeam_channel::Receiver<ChannelPayload>,
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

impl PartialOrd for NodeAndDistance {
    fn partial_cmp(&self, other: &NodeAndDistance) -> Option<std::cmp::Ordering> {
        Some(other.1.cmp(&self.1))
    }
}

impl Ord for NodeAndDistance {
    fn cmp(&self, other: &NodeAndDistance) -> std::cmp::Ordering {
        other.1.cmp(&self.1)
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
    pub fn new(
        node: Node,
        bootstrap: Option<Node>,
        sender: crossbeam_channel::Sender<ChannelPayload>,
        receiver: crossbeam_channel::Receiver<ChannelPayload>,
    ) -> Self {
        let mut kbuckets: Vec<KBucket> = Vec::new();
        for _ in 0..N_BUCKETS {
            kbuckets.push(KBucket::new());
        }

        let mut ret = Self {
            node: node.clone(),
            kbuckets,
            sender,
            receiver,
        };

        ret.update(node);

        if let Some(bootstrap) = bootstrap {
            ret.update(bootstrap);
        }

        ret
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

    fn contact_via_rpc(&self, dst: Node) -> bool {
        if let Err(_) = self
            .sender
            .send(ChannelPayload::Request((network::Request::Ping, dst)))
        {
            println!(
                "[FAILED] RoutingTable::contact_via_rpc --> Receiver is dead, closing channel"
            );
            return false;
        }

        true
    }

    pub fn update(&mut self, node: Node) {
        let bucket_idx = self.get_lookup_bucket_index(&node.id);

        // TODO(testing): fill buckets with dummy nodes so we can reach the else statement
        if self.kbuckets[bucket_idx].nodes.len() < K_PARAM {
            let node_idx = self.kbuckets[bucket_idx]
                .nodes
                .iter()
                .position(|x| x.id == node.id);
            match node_idx {
                Some(i) => {
                    self.kbuckets[bucket_idx].nodes.remove(i);
                    self.kbuckets[bucket_idx].nodes.push(node);
                }
                None => {
                    self.kbuckets[bucket_idx].nodes.push(node);
                }
            }
        } else {
            let _success = self.contact_via_rpc(self.kbuckets[bucket_idx].nodes[0].clone());
            let receiver = self.receiver.clone();

            // We must make it blocking (no threads) because we still want to use the mutable reference to the buckets
            // we don't want to clone and move
            let res = receiver
                .recv()
                .expect("[FAILED] Routing::update --> Failed  to receive data from channel");
            match res {
                ChannelPayload::Response(_) => {
                    let to_re_add = self.kbuckets[bucket_idx].nodes.remove(0);
                    self.kbuckets[bucket_idx].nodes.push(to_re_add);
                }
                ChannelPayload::Request(_) => {
                    eprintln!("[FAILED] Routing::update --> Unexpectedly got a Request instead of a Response");
                }
                ChannelPayload::NoData => {
                    self.kbuckets[bucket_idx].nodes.remove(0);
                    self.kbuckets[bucket_idx].nodes.push(node);
                }
            };
        }
    }

    pub fn remove(&mut self, node: &Node) {
        let bucket_idx = self.get_lookup_bucket_index(&node.id);

        if let Some(i) = self.kbuckets[bucket_idx]
            .nodes
            .iter()
            .position(|x| x.id == node.id)
        {
            self.kbuckets[bucket_idx].nodes.remove(i);
        } else {
            eprintln!("[WARN] Routing::remove --> Tried to remove non-existing entry");
        }
    }

    pub fn get_closest_nodes(&self, key: &Key, count: usize) -> Vec<NodeAndDistance> {
        /*
            Notes:

            Kademlia seems not to specify a nice way to search for K closest nodes

            # Method 1 (currently the best one)
                1) look at which bucket index the key falls into
                2) check buckets higher up that index
                3) check buckets lower down that index
                (of course stop when you reach the desired count)
        */

        let mut ret = Vec::with_capacity(count);

        if count == 0 {
            return ret;
        }

        let mut bucket_index = self.get_lookup_bucket_index(key);
        let mut bucket_index_copy = bucket_index;

        for node in &self.kbuckets[bucket_index].nodes {
            ret.push(NodeAndDistance(node.clone(), Distance::new(&node.id, key)));
        }

        while ret.len() < count && bucket_index < self.kbuckets.len() - 1 {
            bucket_index += 1;

            for node in &self.kbuckets[bucket_index].nodes {
                ret.push(NodeAndDistance(node.clone(), Distance::new(&node.id, key)));
            }
        }

        while ret.len() < count && bucket_index_copy > 0 {
            bucket_index_copy -= 1;

            for node in &self.kbuckets[bucket_index_copy].nodes {
                ret.push(NodeAndDistance(node.clone(), Distance::new(&node.id, key)));
            }
        }

        ret.sort_by(|a, b| a.1.cmp(&b.1));
        ret.truncate(count);
        ret
    }
}
