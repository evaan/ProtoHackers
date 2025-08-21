use std::{io::{BufRead, BufReader, Write}, net::{SocketAddr, TcpListener, TcpStream}, sync::{Arc, Mutex}, thread};

struct Client {
    addr: SocketAddr,
    stream: TcpStream
}

fn handle_client(client: Client, addr: SocketAddr, clients: Arc<Mutex<Vec<Client>>>, names: Arc<Mutex<Vec<String>>>) {
    let mut buf = String::new();
    let mut name = String::new();
    let mut stream = client.stream;
    let _ = stream.write("Welcome to budgetchat! What is your name?\n".as_bytes());
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    loop {
        buf.clear();
        match reader.read_line(&mut buf) {
            Ok(0) | Err(_) => {
                {
                    let mut clients = clients.lock().unwrap();
                    clients.retain(|c| c.addr != addr);
                    if !name.is_empty() {
                        for c in clients.iter_mut() {
                            let _ = c.stream.write(format!("* {} left the chat\n", name).as_bytes());
                        }
                    }
                }
                if !name.is_empty() {
                    let mut names = names.lock().unwrap();
                    if let Some(index) = names.iter().position(|n| *n == name) {
                        names.swap_remove(index);
                    }
                }
                return;
            }
            Ok(_) => {
                if name.is_empty() {
                    name = buf.trim().to_string();
                    if name.is_empty() || !name.chars().all(char::is_alphanumeric) {
                        let _ = stream.write("Please pick a valid name\n".as_bytes());
                        return;
                    }
                    {
                        for c in clients.lock().unwrap().iter_mut() {
                            let _ = c.stream.write(format!("* {} joined the chat\n", name).as_bytes());
                        }
                        let mut clients = clients.lock().unwrap();
                        clients.push(Client { addr: client.addr, stream: stream.try_clone().unwrap() });
                    }
                    {
                        let mut names = names.lock().unwrap();
                        let _ = stream.write(format!("* Users online: {}\n", names.join(", ")).as_bytes());
                        names.push(name.to_string());
                    }
                } else {
                    let msg = buf.trim().to_string();
                    if !msg.is_empty() {
                        let mut clients = clients.lock().unwrap();
                        for c in clients.iter_mut() {
                            if c.addr != addr {
                                let _ = c.stream.write_all(format!("[{}] {}\n", name, msg).as_bytes());
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1111").unwrap();
    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));
    let names: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    loop {
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let clients = Arc::clone(&clients);
            let names = Arc::clone(&names);
            let addr = stream.try_clone().unwrap().peer_addr().unwrap();
            thread::spawn(move || {
                handle_client(Client{addr: addr, stream: stream}, addr, clients, names);
            });
        }
    }
}
