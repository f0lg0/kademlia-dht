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
            .expect("[FAILED] Rpc::new --> Error while binding UdpSocket to specified addr");

        Self {
            socket: Arc::new(socket),
            pending: Arc::new(Mutex::new(HashMap::new())),
            node,
        }
    }
    pub fn open(rpc: Rpc, sender: mpsc::Sender<ReqWrapper>) {
        thread::spawn(move || {
            let mut buf = [0u8; BUF_SIZE];

            loop {
                let (_, src_addr) = rpc
                    .socket
                    .recv_from(&mut buf)
                    .expect("[FAILED] Rpc::open --> Failed to receive data from peer");

                let msg_len: u32 = str::from_utf8(&buf)
                    .expect("[!] Failed to parse message length.")
                    .parse::<u32>()
                    .expect("[!] Failed to parse message length from string to u32");

                println!("\ngot msg size: {}", msg_len);

                let mut end_buf = Vec::with_capacity((msg_len + 1) as usize);
                for _ in 0..(msg_len + 1) {
                    end_buf.push(0x00);
                }

                // todo: the problem is that we dont continue straming the same msg after the first 4 bytes
                let len = rpc
                    .socket
                    .recv(&mut end_buf)
                    .expect("[FAILED] Rpc::open --> Failed to receive data from peer");

                let payload =
                    String::from(str::from_utf8(&end_buf[..len]).expect(
                        "[FAILED] Rpc::open --> Unable to parse string from received bytes",
                    ));
                println!("\ndecoded: {}", payload);

                let mut decoded: RpcMessage = serde_json::from_str(&payload)
                    .expect("[FAILED] Rpc::open, serde_json --> Unable to decode string payload");

                decoded.src = src_addr.to_string();

                if super::VERBOSE {
                    println!(
                        "----------\n[+] Received message: {:?}\n\ttoken: {:?}\n\tsrc: {}\n\tdst: {}\n\tmsg: {:?}\n----------",
                        &decoded.msg, &decoded.token, &decoded.src, &decoded.dst, &decoded.msg
                    );
                }

                if decoded.dst != rpc.node.get_addr() {
                    eprintln!("[WARNING] Rpc::open --> Destination address doesn't match node address, ignoring");
                    continue;
                }

                match decoded.msg {
                    Message::Abort => {
                        break;
                    }
                    Message::Request(req) => {
                        let wrapped_req = ReqWrapper {
                            token: decoded.token,
                            src: decoded.src,
                            payload: req,
                        };

                        if let Err(_) = sender.send(wrapped_req) {
                            eprintln!("[FAILED] Rpc::open, Request --> Receiver is dead, closing channel.");
                            break;
                        }
                    }
                    Message::Response(res) => {
                        rpc.clone().handle_response(decoded.token, res);
                    }
                }
            }
        });
    }

    pub fn send_msg(&self, msg: &RpcMessage) {
        let encoded = serde_json::to_string(msg)
            .expect("[FAILED] Rpc::send_msg --> Unable to serialize message");

        let final_msg = format!("{:04}{}", encoded.len(), encoded);
        println!("sending: {}", final_msg);

        self.socket
            .send_to(&final_msg.as_bytes(), &msg.dst)
            .expect("[FAILED] Rpc::send_msg --> Error while sending message to specified address");
    }

    pub fn handle_response(self, token: Key, res: Response) {
        thread::spawn(move || {
            let mut pending = self
                .pending
                .lock()
                .expect("[FAILED] Rpc::handle_response --> Failed to acquire lock on Pending");

            let tmp = match pending.get(&token) {
                Some(sender) => sender.send(Some(res)),
                None => {
                    eprintln!(
                        "[WARNING] Rpc::handle_response --> Unsolicited response received, ignoring..."
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
            .expect("[FAILED] Rpc::make_request --> Failed to acquire mutex on Pending");

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
                    .expect("[FAILED] Rpc::make_request --> Failed to acquire mutex on Pending");
                pending.remove(&token);
            }
        });

        receiver
    }
}
