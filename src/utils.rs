use super::protocol::Protocol;
use std::fs::create_dir;
use std::io::Write;
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

pub fn dump_interface_state(interface: &Protocol, path: &str) {
    create_dir("dumps").expect("utils::dump_interface_state --> Unable to create dumps dir");

    let rt = interface.routes.lock().unwrap();
    let st = interface.store.lock().unwrap();

    let json = serde_json::json!({
        "node": {
            "ip": interface.node.ip,
            "port": interface.node.port,
            "id": format!("{:?}", interface.node.id),
        },
        "routes": {
            "node": *rt.node.get_info(),
            "kbuckets": format!("{:?}", *rt),
        },
        "store": format!("{:?}", *st),
        "rpc": {
            "socket": format!("{:?}", interface.rpc.socket),
            "pending": format!("{:?}", interface.rpc.pending.lock().unwrap()),
            "node": interface.rpc.node.get_info(),
        }
    });

    let mut file = std::fs::File::create(path)
        .expect("utils::dump_interface_state --> Unable to create dump file");
    file.write_all(&json.to_string().as_bytes())
        .expect("utils::dump_interface_state --> Unable to write to dump file");
}
