// use super::node::ID;
use std::net::UdpSocket;

pub fn get_local_ip() -> Option<String> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None,
    };

    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None,
    };

    match socket.local_addr() {
        Ok(addr) => return Some(addr.ip().to_string()),
        Err(_) => return None,
    };
}

// pub fn xor_distance(id0: &ID, id1: &ID) -> [u8; 32] {
//     let bytes0 = id0.as_bytes();
//     let bytes1 = id1.as_bytes();

//     let mut ret: [u8; 32];
//     for i in 0..bytes0.len() {
//         ret.push(bytes0[i] ^ bytes1[i]);
//     }

//     ret
// }
