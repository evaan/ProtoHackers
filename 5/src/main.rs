use std::{io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, thread};

fn proxy(mut client: TcpStream, remote: TcpStream) {
    let mut reader = BufReader::new(remote.try_clone().unwrap());
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => {
                let _ = client.shutdown(std::net::Shutdown::Write);
            },
            Ok(_) => {
                let (body, eol) = if line.ends_with("\r\n") {
                    (&line[..line.len()-2], "\r\n")
                } else if line.ends_with('\n') {
                    (&line[..line.len()-1], "\n")
                } else {
                    (line.as_str(), "")
                };
                let mut result: Vec<String> = Vec::new();
                for part in body.split(' ') {
                    if part.len() >= 26 && part.len() <= 35 && part.starts_with("7") {
                        result.push("7YWHMfk9JZe0LM0g1ZauHuiSxhI".to_string());
                    } else {
                        result.push(part.to_string());
                    }
                }
                let new_msg = result.join(" ") + eol;
                let _ = client.write(new_msg.as_bytes());
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1111").unwrap();

    loop {
        for client in listener.incoming() {
            let client = client.unwrap();
            let remote = TcpStream::connect("chat.protohackers.com:16963").unwrap();
            let client_clone = client.try_clone().unwrap();
            let remote_clone = remote.try_clone().unwrap();
            thread::spawn(move || { proxy(client, remote_clone); });
            thread::spawn(move || { proxy(remote, client_clone); });
        }
    }
}
