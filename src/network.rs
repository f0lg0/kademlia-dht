use serde::{Deserialize, Serialize};

use super::key::Key;
use super::node::*;
use super::routing::FindValueResult;
use super::routing::NodeAndDistance;
use super::BUF_SIZE;
use super::TIMEOUT;

use std::collections::HashMap;
use std::net::UdpSocket;
use std::str;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Ping,
    Store(String, String),
    FindNode(Key),
    FindValue(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Ping,
    FindNode(Vec<NodeAndDistance>),
    FindValue(FindValueResult),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Abort,
    Request(Request),
    Response(Response),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcMessage {
    pub token: Key,
    pub src: String,
    pub dst: String,
    pub msg: Message,
}

#[derive(Debug)]
pub struct ReqWrapper {
    pub token: Key,
    pub src: String,
    pub payload: Request,
}

#[derive(Clone, Debug)]
pub struct Rpc {
    pub socket: Arc<UdpSocket>,
    pub pending: Arc<Mutex<HashMap<Key, mpsc::Sender<Option<Response>>>>>,
    pub node: Node,
}

impl Rpc {
    pub fn new(node: Node) -> Self {
        let socket = UdpSocket::bind(node.get_addr())
            .expect("Rpc::new --> Error while binding UdpSocket to specified addr");

        Self {
            socket: Arc::new(socket),
            pending: Arc::new(Mutex::new(HashMap::new())),
            node,
        }
    }
    pub fn open(rpc: Rpc, sender: mpsc::Sender<ReqWrapper>) {
        println!("[*] Network::open --> Listening on {:?}", &rpc.socket);

        thread::spawn(move || {
            let mut buf = [0u8; BUF_SIZE];

            loop {
                let (len, src_addr) = rpc
                    .socket
                    .recv_from(&mut buf)
                    .expect("Rpc::open --> Failed to receive data from peer");

                let payload = String::from(
                    str::from_utf8(&buf[..len])
                        .expect("Rpc::open --> Unable to parse string from received bytes"),
                );

                let mut decoded: RpcMessage = serde_json::from_str(&payload)
                    .expect("Rpc::open, serde_json --> Unable to decode string payload");

                decoded.src = src_addr.to_string();

                println!(
                    "----------\n[+] Received message: {:?}\n\ttoken: {:?}\n\tsrc: {}\n\tdst: {}\n\tmsg: {:?}\n----------",
                    &decoded.msg, &decoded.token, &decoded.src, &decoded.dst, &decoded.msg
                );

                if decoded.dst != rpc.node.get_addr() {
                    println!("[!] Destination address doesn't match node address, ignoring [!]");
                    continue;
                }

                match decoded.msg {
                    Message::Abort => {
                        break;
                    }
                    Message::Request(req) => {
                        println!("Request content: {:?}", &req);
                        let wrapped_req = ReqWrapper {
                            token: decoded.token,
                            src: decoded.src,
                            payload: req,
                        };

                        if let Err(_) = sender.send(wrapped_req) {
                            println!("Rpc::open, Request --> Receiver is dead, closing channel.");
                            break;
                        }
                    }
                    Message::Response(res) => {
                        println!("Response content: {:?}", res);
                        rpc.clone().handle_response(decoded.token, res);
                    }
                }
            }
        });
    }

    pub fn send_msg(&self, msg: &RpcMessage) {
        let encoded =
            serde_json::to_string(msg).expect("Rpc::send_msg --> Unable to serialize message");
        self.socket
            .send_to(&encoded.as_bytes(), &msg.dst)
            .expect("Rpc::send_msg --> Error while sending message to specified address");

        println!(
            "[+] Network::send_msg --> From: {}, To: {}, Token: {:?}",
            &msg.src, &msg.dst, &msg.token
        );
    }

    pub fn handle_response(self, token: Key, res: Response) {
        thread::spawn(move || {
            let mut pending = self
                .pending
                .lock()
                .expect("Failed to acquire lock on 'Pending' struct");

            let tmp = match pending.get(&token) {
                Some(sender) => sender.send(Some(res)),
                None => {
                    eprintln!(
                        "Rpc::handle_response --> Unsolicited response received, ignoring..."
                    );
                    return;
                }
            };

            if let Ok(_) = tmp {
                pending.remove(&token);
            }
        });
    }

    pub fn make_request(&self, req: Request, dst: Node) -> mpsc::Receiver<Option<Response>> {
        let (sender, receiver) = mpsc::channel();
        let mut pending = self
            .pending
            .lock()
            .expect("Failed to acquire mutex on 'Pending' struct");

        let token = Key::new(format!(
            "{}:{}:{:?}",
            self.node.get_info(),
            dst.get_info(),
            std::time::SystemTime::now()
        ));
        pending.insert(token.clone(), sender.clone());

        let msg = RpcMessage {
            token: token.clone(),
            src: self.node.get_addr(),
            dst: dst.get_addr(),
            msg: Message::Request(req),
        };

        self.send_msg(&msg);

        let rpc = self.clone();
        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(TIMEOUT));
            if let Ok(_) = sender.send(None) {
                let mut pending = rpc
                    .pending
                    .lock()
                    .expect("Failed to acquire mutex on 'Pending' struct");
                pending.remove(&token);
            }
        });

        receiver
    }
}
