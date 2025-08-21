use core::time;
use std::{io::{Read, Write}, net::{SocketAddr, TcpListener, TcpStream}, sync::{Arc, Mutex}, thread::{self, sleep}};

struct HeartbeatStream {
    time: u32,
    stream: TcpStream,
    addr: SocketAddr
}

struct CameraStream {
    road: u16,
    mile: u16,
    limit: u16,
    stream: TcpStream,
    addr: SocketAddr
}

struct DispatchStream {
    roads: Vec<u16>,
    stream: TcpStream,
    addr: SocketAddr
}

#[derive(Clone)]
struct Plate {
    plate: String,
    time: u32,
    mile: u16,
    road: u16,
    limit: u16
}

struct Ticket {
    plate: String,
    day: u32
}

fn handle_client(mut stream: TcpStream, addr: SocketAddr, heartbeat_streams: Arc<Mutex<Vec<HeartbeatStream>>>, camera_streams: Arc<Mutex<Vec<CameraStream>>>, dispatch_streams: Arc<Mutex<Vec<DispatchStream>>>, plate_times: Arc<Mutex<Vec<Plate>>>, tickets: Arc<Mutex<Vec<Ticket>>>) {
    let mut data = [0; 1];
    loop {
        match stream.read(&mut data) {
            Ok(0) | Err(_) => {
                {
                    let mut heartbeat_streams = heartbeat_streams.lock().unwrap();
                    heartbeat_streams.retain(|hs| hs.addr != addr);
                }
                {
                    let mut camera_streams = camera_streams.lock().unwrap();
                    camera_streams.retain(|cs| cs.addr != addr);
                }
                {
                    let mut dispatch_streams = dispatch_streams.lock().unwrap();
                    dispatch_streams.retain(|ds| ds.addr != addr);
                }
                return;
            }
            Ok(_) => {
                let mut stream = stream.try_clone().unwrap();
                match data[0] {
                    0x40 => {
                        let mut buf = [0; 4];
                        let _ = stream.read(&mut buf);
                        let time = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
                        {
                            let mut heartbeat_streams = heartbeat_streams.lock().unwrap();
                            let addr = stream.peer_addr().unwrap();
                            for s in heartbeat_streams.iter() {
                                if s.addr == addr {
                                    let _ = stream.write(&[0x10, 0x03, 0x62, 0x61, 0x64]);
                                    return;
                                }
                            }
                            heartbeat_streams.push(HeartbeatStream { time: time, stream: stream, addr: addr });
                        }
                    }
                    0x80 => {
                        let mut buf = [0; 6];
                        let _ = stream.read(&mut buf);
                        let road = u16::from_be_bytes([buf[0], buf[1]]);
                        let mile = u16::from_be_bytes([buf[2], buf[3]]);
                        let limit = u16::from_be_bytes([buf[4], buf[5]]);
                        {
                            let mut camera_streams = camera_streams.lock().unwrap();
                            for s in camera_streams.iter() {
                                if s.addr == addr {
                                    let _ = stream.write(&[0x10, 0x03, 0x62, 0x61, 0x64]);
                                    return;
                                }
                            }
                            camera_streams.push(CameraStream { road: road, mile: mile, limit: limit, stream: stream, addr: addr });
                        }
                        {
                            let mut dispatch_streams = dispatch_streams.lock().unwrap();
                            for s in dispatch_streams.iter_mut() {
                                if s.addr == addr {
                                    let _ = s.stream.write(&[0x10, 0x03, 0x62, 0x61, 0x64]);
                                    return;
                                }
                            }
                        }
                    }
                    0x81 => {
                        let mut buf = [0; 1];
                        let _ = stream.read(&mut buf);
                        let size = buf[0];
                        let mut roads: Vec<u16> = Vec::new();
                        for _ in 0..size {
                            let mut buf = [0; 2];
                            match stream.read(&mut buf) {
                                Ok(0) | Err(_) => {
                                    let _ = stream.write(&[0x10, 0x03, 0x62, 0x61, 0x64]);
                                    return;
                                },
                                Ok(_) => {
                                    roads.push(u16::from_be_bytes([buf[0], buf[1]]));
                                },
                            } 
                        }
                        let roads_clone = roads.clone();
                        {
                            let mut dispatch_streams = dispatch_streams.lock().unwrap();
                            for s in dispatch_streams.iter() {
                                if s.addr == addr {
                                    let _ = stream.write(&[0x10, 0x03, 0x62, 0x61, 0x64]);
                                    return;
                                }
                            }
                            dispatch_streams.push(DispatchStream { roads: roads, stream: stream, addr: addr });
                            drop(dispatch_streams);
                        }
                        {
                            let mut camera_streams = camera_streams.lock().unwrap();
                            for s in camera_streams.iter_mut() {
                                if s.addr == addr {
                                    let _ = s.stream.write(&[0x10, 0x03, 0x62, 0x61, 0x64]);
                                    return;
                                }
                            }
                            drop(camera_streams);
                        }
                        let plate_times1 = plate_times.lock().unwrap();
                        let plates: Vec<Plate> = plate_times1.iter().cloned().collect();
                        drop(plate_times1);
                        for plate in plates {
                            if roads_clone.contains(&plate.road) {
                                handle_reading(plate.plate.clone(), plate.road, plate.limit, &dispatch_streams, &plate_times, &tickets);
                            }
                        }
                    }
                    0x20 => {
                        let mut buf = [0; 1];
                        let _ = stream.read(&mut buf);
                        let mut plate_vec: Vec<u16> = Vec::new();
                        for _ in 0..buf[0] {
                            let mut buf = [0; 1];
                            match stream.read(&mut buf) {
                                Ok(0) | Err(_) => {
                                    let _ = stream.write(&[0x10, 0x03, 0x62, 0x61, 0x64]);
                                    return;
                                },
                                Ok(_) => {
                                    plate_vec.push(buf[0].into());
                                },
                            } 
                        }
                        let plate = String::from_utf16_lossy(&plate_vec);
                        let mut road: u16 = 0;
                        let mut mile: u16 = 0;
                        let mut limit: u16 = 0;
                        let mut time_buf = [0; 4];
                        let _ = stream.read(&mut time_buf);
                        {
                            let camera_streams = camera_streams.lock().unwrap();
                            let mut found = false;
                            for s in camera_streams.iter() {
                                if s.addr == addr {
                                    found = true;
                                    road = s.road;
                                    mile = s.mile;
                                    limit = s.limit;
                                }
                            }
                            if !found {
                                let _ = stream.write(&[0x10, 0x03, 0x62, 0x61, 0x64]);
                                return; 
                            }
                        }
                        {
                            let mut plate_times = plate_times.lock().unwrap();
                            plate_times.push(Plate { plate: plate.to_string(), time: u32::from_be_bytes(time_buf), road: road, mile: mile, limit: limit });
                        }
                        {
                            handle_reading(plate.to_string(), road, limit, &dispatch_streams, &plate_times, &tickets);
                        }
                    }
                    _ => {
                        let _ = stream.write(&[0x10, 0x03, 0x62, 0x61, 0x64]);
                        return;
                    }
                }
            }
        }
    }
}

fn heartbeats(heartbeat_streams: Arc<Mutex<Vec<HeartbeatStream>>>) {
    let mut counter: u128 = 0;
    loop {
        {
            let mut heartbeat_streams = heartbeat_streams.lock().unwrap();
            for hbs in heartbeat_streams.iter_mut() {
                if hbs.time != 0 && counter % (hbs.time as u128) == 0 {
                    let _ = hbs.stream.write(&[0x41]);
                }
            }
        }
        sleep(time::Duration::from_millis(100));
        counter += 1;
    }
}

fn handle_reading(plate: String, road: u16, limit: u16, dispatch_streams: &Arc<Mutex<Vec<DispatchStream>>>, plate_times: &Arc<Mutex<Vec<Plate>>>, tickets: &Arc<Mutex<Vec<Ticket>>>) {
    let mut relevant_readings: Vec<Plate> = {
        let pts = plate_times.lock().unwrap();
        pts.iter()
            .filter(|r| r.road == road && r.plate == plate)
            .cloned()
            .collect()
    };
    relevant_readings.sort_unstable_by_key(|r| r.time);
    if relevant_readings.len() < 2 {
        return;
    }
    for w in relevant_readings.windows(2) {
        let a = &w[0];
        let b = &w[1];
        let dt = b.time - a.time;
        if dt == 0 {
            continue;
        }
        let dm = (b.mile as i32 - a.mile as i32).unsigned_abs() as u32;
        let raw = (dm as u64).saturating_mul(360000) / (dt as u64);
        let speed: u16 = raw.min(u16::MAX as u64) as u16;
        if (speed as u32) <= (limit as u32) * 100 {
            continue;
        }
        let day1 = a.time / 86400;
        let day2 = b.time / 86400;
        {

        }
        let mut dispatched = false;
        {
            let mut ds = dispatch_streams.lock().unwrap();
            for s in ds.iter_mut() {
                if s.roads.contains(&road) {
                    let mut msg: Vec<u8> = Vec::new();
                    msg.push(0x21);
                    msg.push(plate.len() as u8);
                    msg.extend_from_slice(plate.as_bytes());
                    msg.extend_from_slice(&road.to_be_bytes());
                    msg.extend_from_slice(&a.mile.to_be_bytes());
                    msg.extend_from_slice(&a.time.to_be_bytes());
                    msg.extend_from_slice(&b.mile.to_be_bytes());
                    msg.extend_from_slice(&b.time.to_be_bytes());
                    msg.extend_from_slice(&speed.to_be_bytes());
                    let tickets_guard = tickets.lock().unwrap();
                    let mut found = false;
                    for t in tickets_guard.iter() {
                        if t.plate == plate && (t.day == day1 || t.day == day2) {
                            found = true;
                            break;
                        }
                    }
                    if found {
                        continue;
                    }
                    if s.stream.write_all(&msg).is_ok() {
                        dispatched = true;
                    }
                    break;
                }
            }
        }
        if dispatched {
            let mut tickets = tickets.lock().unwrap();
            let plate_clone = plate.clone();
            tickets.push(Ticket { plate: plate_clone.clone(), day: day1 as u32 });
            if day1 != day2 {
                tickets.push(Ticket { plate: plate_clone, day: day2 as u32 });
            }
            return;
        }
    }
}

fn main() {
    let heartbeat_streams: Arc<Mutex<Vec<HeartbeatStream>>> = Arc::new(Mutex::new(Vec::new()));
    let camera_streams: Arc<Mutex<Vec<CameraStream>>> = Arc::new(Mutex::new(Vec::new()));
    let dispatch_streams: Arc<Mutex<Vec<DispatchStream>>> = Arc::new(Mutex::new(Vec::new()));
    let plate_times: Arc<Mutex<Vec<Plate>>> = Arc::new(Mutex::new(Vec::new()));
    let tickets: Arc<Mutex<Vec<Ticket>>> = Arc::new(Mutex::new(Vec::new()));
    let listener = TcpListener::bind("0.0.0.0:1111").unwrap();
    {
        let heartbeat_streams = Arc::clone(&heartbeat_streams);
        thread::spawn(move || { heartbeats(heartbeat_streams); });
    }
    loop {
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let addr = stream.peer_addr().unwrap();
            let heartbeat_streams = Arc::clone(&heartbeat_streams);
            let camera_streams = Arc::clone(&camera_streams);
            let dispatch_streams = Arc::clone(&dispatch_streams);
            let plate_times = Arc::clone(&plate_times);
            let tickets = Arc::clone(&tickets);
            thread::spawn(move || { handle_client(stream, addr, heartbeat_streams, camera_streams, dispatch_streams, plate_times, tickets); });
        }
    }
}
