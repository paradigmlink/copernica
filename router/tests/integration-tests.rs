extern crate packets;

use async_std::net::UdpSocket;
use async_std::task;
use async_std::future;
use packets::{Packet, request};
use bincode::{serialize, deserialize};

fn main() {
    let send = async {
        let socket = UdpSocket::bind("127.0.0.1:8081").await.unwrap();

        let add0 = "127.0.0.1:8090";
        let msg0 = request("oa".to_string());
        let msg0s = serialize(&msg0).unwrap();
        let add1 = "127.0.0.1:8092";
        let msg1 = request("world".to_string());
        let msg1s = serialize(&msg1).unwrap();
        let add2 = "127.0.0.1:8094";
        let msg2 = request("hello world".to_string());
        let msg2s = serialize(&msg2).unwrap();
        let add3 = "127.0.0.1:8096";
        let msg3 = request("woah".to_string());
        let msg3s = serialize(&msg3).unwrap();
        let add4 = "127.0.0.1:8098";
        let msg4 = request("woah2".to_string());
        let msg4s = serialize(&msg4).unwrap();
        socket.send_to(&msg0s, &add0).await.unwrap();
        println!("UDP SENT {}>{}:{:?}",socket.local_addr().unwrap(), add0, &msg0);
        socket.send_to(&msg1s, &add1).await.unwrap();
        println!("UDP SENT {}>{}:{:?}",socket.local_addr().unwrap(), add1, &msg1);
        socket.send_to(&msg2s, &add2).await.unwrap();
        println!("UDP SENT {}>{}:{:?}",socket.local_addr().unwrap() ,add2, &msg2);
        socket.send_to(&msg3s, &add3).await.unwrap();
        println!("UDP SENT {}>{}:{:?}",socket.local_addr().unwrap() ,add3, &msg3);
        socket.send_to(&msg4s, &add4).await.unwrap();
        println!("UDP SENT {}>{}:{:?}",socket.local_addr().unwrap() ,add4, &msg4);
    };
    let recv0 = async {
        let socket = UdpSocket::bind("127.0.0.1:8091").await.unwrap();
        let mut buf = vec![0u8; 1024];
        let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
        let packet: Packet = deserialize(&buf[..n]).unwrap();
        println!("UDP RECV {}>{}:{:?}", peer,socket.local_addr().unwrap(), packet);
    };
    let recv1 = async {
        let socket = UdpSocket::bind("127.0.0.1:8093").await.unwrap();
        let mut buf = vec![0u8; 1024];
        let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
        let packet: Packet = deserialize(&buf[..n]).unwrap();
        println!("UDP RECV {}>{}:{:?}", peer,socket.local_addr().unwrap(), packet);
    };
    let recv2 = async {
        let socket = UdpSocket::bind("127.0.0.1:8095").await.unwrap();
        let mut buf = vec![0u8; 1024];
        let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
        let packet: Packet = deserialize(&buf[..n]).unwrap();
        println!("UDP RECV {}>{}:{:?}", peer, socket.local_addr().unwrap(), packet);
    };
    let recv3 = async {
        let socket = UdpSocket::bind("127.0.0.1:8097").await.unwrap();
        let mut buf = vec![0u8; 1024];
        let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
        let packet: Packet = deserialize(&buf[..n]).unwrap();
        println!("UDP RECV {}>{}:{:?}", peer, socket.local_addr().unwrap(), packet);
    };
    let recv4 = async {
        let socket = UdpSocket::bind("127.0.0.1:8099").await.unwrap();
        let mut buf = vec![0u8; 1024];
        let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
        let packet: Packet = deserialize(&buf[..n]).unwrap();
        println!("UDP RECV {}>{}:{:?}", peer, socket.local_addr().unwrap(), packet);
    };
    let joined = future::join!(recv0, recv1, recv2, recv3, recv4, send);
    task::block_on(joined);
}
