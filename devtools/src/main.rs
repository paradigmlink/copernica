use std::thread;
use std::net;
use packets::{Packet, mk_data, mk_interest};
use bincode::{serialize, deserialize};

fn socket(listen_on: net::SocketAddr) -> net::UdpSocket {
    let attempt = net::UdpSocket::bind(listen_on);
    let mut socket;
    match attempt {
        Ok(sock) => {
            println!("Bound socket to {}", listen_on);
            socket = sock;
        },
        Err(err) => panic!("Could not bind: {}", err)
    }
    socket
}

fn read_message(socket: net::UdpSocket) -> Vec<u8> {
    let mut buf: [u8; 200] = [0; 200];
    println!("Reading data");
    let result = socket.recv_from(&mut buf);
    drop(socket);
    let mut data;
    match result {
        Ok((amt, src)) => {
            println!("Received data from {}", src);
            data = Vec::from(&buf[0..amt]);
        },
        Err(err) => panic!("Read error: {}", err)
    }
    data
}

pub fn send_message(send_addr: net::SocketAddr, target: net::SocketAddr, data: Vec<u8>) {
    let socket = socket(send_addr);
    println!("Sending data to {}", &target);
    let result = socket.send_to(&data, target);
    drop(socket);
    match result {
        Ok(amt) => println!("Sent {} bytes", amt),
        Err(err) => panic!("Write error: {}", err)
    }
}

pub fn listen(listen_on: net::SocketAddr) -> thread::JoinHandle<Vec<u8>> {
    let socket = socket(listen_on);
    let handle = thread::spawn(move || {
        read_message(socket)
    });
    handle
}

fn main() {
    println!("UDP");
    let ip = net::Ipv4Addr::new(127, 0, 0, 1);
    let listen_addr = net::SocketAddrV4::new(ip, 8090);
    let send_addr = net::SocketAddrV4::new(ip, 8092);
    //let future = listen(net::SocketAddr::V4(listen_addr));

    let interest = mk_interest("hello/how/are/you/today?".to_string());
    let message = serialize(&interest);
    //let message: Vec<u8> = vec![10];
    // give the thread 3s to open the socket
        //thread::sleep_ms(3000);
    send_message(net::SocketAddr::V4(send_addr), net::SocketAddr::V4(listen_addr), message.unwrap());
    //println!("Waiting");
    //let received = future.join().unwrap();
    //println!("Got {} bytes", received.len());
}

