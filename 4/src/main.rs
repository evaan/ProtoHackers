use std::{collections::HashMap, net::UdpSocket};

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:1111").unwrap();
    let mut buf = [0; 1024];
    let mut data: HashMap<String, String> = HashMap::new();
    data.insert("version".to_string(), "evans awesome database v1".to_string());
    loop {
        let (n, src) = socket.recv_from(&mut buf).unwrap();
        let msg = String::from_utf8_lossy(&buf[..n]).to_string();
        if msg.contains("=") {
            if let Some(eq) = msg.find('=') {
                let (k, v) = msg.split_at(eq);
                let v = &v[1..];
                if k != "version" {
                    data.insert(k.to_string(), v.to_string());
                }
            }
        } else if let Some(val) = data.get(&msg) {
            let _ = socket.send_to(format!("{}={}", msg, val).as_bytes(), &src);
        }
    }
}
