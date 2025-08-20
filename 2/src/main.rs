use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, thread};

struct PriceAndTime {
    timestamp: i32,
    price: i32,
}

fn handle_client(mut stream: TcpStream) {
    let mut prices: Vec<PriceAndTime> = Vec::new();
    loop {
        let mut buf = [0; 9];
        while match stream.peek(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    return;
                }
                n != 9
            }
            Err(_) => {
                return;
            }
        } {}
        match stream.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    return;
                }
                if buf[0] == b'I' {
                    let timestamp = i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                    let price = i32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]);
                    prices.push(PriceAndTime{timestamp: timestamp, price: price});
                }
                if buf[0] == b'Q' {
                    let min_time = i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                    let max_time = i32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]);
                    let mut total: i128 = 0;
                    let mut hits = 0;
                    for price in &prices {
                        if min_time <= price.timestamp && price.timestamp <= max_time {
                            total += price.price as i128;
                            hits += 1;
                        }
                    }
                    // 0/0 bad 0/1 good
                    if hits == 0 {
                        hits = 1;
                    }
                    let _ = stream.write(&i32::to_be_bytes((total/hits) as i32));
                }
            }
            Err(_) => {
                return;
            }
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
