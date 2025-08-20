use std::{io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, thread};

use serde_json::Value;

fn is_prime(num: u128) -> bool {
    if num <= 1 {
        return false;
    }
    
    for i in 2..=(num as f64).sqrt() as u128 {
        if num % i == 0 {
            return false;
        }
    }

    true
}

fn handle_client(mut stream: TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if line.trim().is_empty() {
            continue;
        }
        let v: Value = match serde_json::from_str(&line) {
            Ok(v) => { v },
            Err(_) => {
                let _ = stream.write(b"{\"error\":true}\n");
                break;
            }
        };
        if (!v["method"].is_string() || v["method"].as_str().unwrap() != "isPrime") || !v["number"].is_number() {
            let _ = stream.write(b"{\"error\":true}\n");
            break;
        }
        if !(v["number"].as_number().unwrap().is_i64() || v["number"].as_number().unwrap().is_u64()) || v["number"].as_number().unwrap().as_i128().unwrap() <= 0 || !is_prime(v["number"].as_number().unwrap().as_u128().unwrap()) {
            let _ = stream.write(b"{\"method\":\"isPrime\",\"prime\":false}\n");
        } else {
            let _ = stream.write(b"{\"method\":\"isPrime\",\"prime\":true}\n");
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1111").unwrap();
    loop {
        for stream in listener.incoming() {
            thread::spawn(move || { handle_client(stream.unwrap()); });
        }
    }
}
