use super::protocol::Protocol;
use std::fs::create_dir_all;
use std::io::Write;
use std::net::UdpSocket;

use super::routing::KBucket;

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
    create_dir_all("dumps").expect("utils::dump_interface_state --> Unable to create dumps dir");

    let rt = interface
        .routes
        .lock()
        .expect("Failed to acquire mutex on 'Routes' struct");
    let st = interface
        .store
        .lock()
        .expect("Failed to acquire mutex on 'Store' struct");

    let flattened: Vec<&KBucket> = rt.kbuckets.iter().collect();

    let mut parsed_buckets = vec![];
    for kb in flattened {
        for n in &kb.nodes {
            let kbucket = serde_json::json!({
                "nodes": {
                    "ip": n.ip,
                    "port": n.port,
                    "id": format!("{:?}", interface.node.id),
                },
                "size": kb.size,
            });
            parsed_buckets.push(kbucket);
        }
    }

    let json = serde_json::json!({
        "node": {
            "ip": interface.node.ip,
            "port": interface.node.port,
            "id": format!("{:?}", interface.node.id),
        },
        "routes": {
            "node": {
                "ip": rt.node.ip,
                "port": rt.node.port,
                "id": format!("{:?}", interface.node.id),
            },
            "kbuckets": parsed_buckets,
        },
        "store": format!("{:?}", *st),
        "rpc": {
            "socket": format!("{:?}", interface.rpc.socket),
            "pending": format!("{:?}", interface.rpc.pending.lock().unwrap()),
            "node": {
                "ip": interface.rpc.node.ip,
                "port": interface.rpc.node.port,
                "id": format!("{:?}", interface.rpc.node.id),
            },
        }
    });

    // write to json file
    let mut file = std::fs::File::create(path)
        .expect("utils::dump_interface_state --> Unable to create dump file");
    file.write_all(&json.to_string().as_bytes())
        .expect("utils::dump_interface_state --> Unable to write to dump file");

    // write also to a .plantuml file
    let mut diagram = std::fs::File::create(format!("{}.plantuml", path))
        .expect("utils::dump_interface_state --> Unable to create dump file");
    diagram
        .write_all("@startjson\n".to_string().as_bytes())
        .expect("utils::dump_interface_state --> Unable to write to dump file");

    diagram
        .write_all(&json.to_string().as_bytes())
        .expect("utils::dump_interface_state --> Unable to write to dump file");

    diagram
        .write_all("\n@endjson".to_string().as_bytes())
        .expect("utils::dump_interface_state --> Unable to write to dump file");
}
