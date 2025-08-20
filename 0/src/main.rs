use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};

fn handle_client(mut stream: TcpStream) {
    let mut data = Vec::new();
    stream.read_to_end(&mut data).expect("Failed to read input stream");
    stream.write_all(&data).expect("Failed to write to output stream");
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1111").expect("Bind failed");

    loop {
        for stream in listener.incoming() {
            handle_client(stream.expect("Failed to access stream"));
        }
    }
}
