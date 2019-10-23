extern crate packets;

use async_std::io;
use async_std::net::UdpSocket;
use async_std::task;
use async_std::future;
use packets::{Packet, mk_interest};
use bincode::{serialize, deserialize};
use std::{thread, time};

fn main() {
    let send = async {
        let socket = UdpSocket::bind("127.0.0.1:8081").await.unwrap();

        let add0 = "127.0.0.1:8090";
        let msg0 = mk_interest("oa".to_string());
        let msg0s = serialize(&msg0).unwrap();
        let add1 = "127.0.0.1:8092";
        let msg1 = mk_interest("world".to_string());
        let msg1s = serialize(&msg1).unwrap();
        let add2 = "127.0.0.1:8094";
        let msg2 = mk_interest("hello world".to_string());
        let msg2s = serialize(&msg2).unwrap();
        socket.send_to(&msg0s, &add0).await.unwrap();
        println!("UDP SENT {}>{}:{:?}",socket.local_addr().unwrap(), add0,  &msg0);
        socket.send_to(&msg1s, &add1).await.unwrap();
        println!("UDP SENT {}>{}:{:?}",socket.local_addr().unwrap(), add1, &msg1);
        socket.send_to(&msg2s, &add2).await.unwrap();
        println!("UDP SENT {}>{}:{:?}",socket.local_addr().unwrap() ,add2, &msg2);
    };
    let recv0 = async {
        let socket1 = UdpSocket::bind("127.0.0.1:8091").await.unwrap();
        let mut buf = vec![0u8; 1024];
        let (n, peer) = socket1.recv_from(&mut buf).await.unwrap();
        let packet: Packet = deserialize(&buf[..n]).unwrap();
        println!("UDP RECV {}>{}:{:?}", peer,socket1.local_addr().unwrap(), packet);
    };


    let recv1 = async {
        let socket2 = UdpSocket::bind("127.0.0.1:8093").await.unwrap();
        let mut buf = vec![0u8; 1024];
        let (n, peer) = socket2.recv_from(&mut buf).await.unwrap();
        let packet: Packet = deserialize(&buf[..n]).unwrap();
        println!("UDP RECV {}>{}:{:?}", peer,socket2.local_addr().unwrap(), packet);
    };


    let recv2 = async {
        let socket3 = UdpSocket::bind("127.0.0.1:8095").await.unwrap();
        let mut buf = vec![0u8; 1024];
        let (n, peer) = socket3.recv_from(&mut buf).await.unwrap();
        let packet: Packet = deserialize(&buf[..n]).unwrap();
        println!("UDP RECV {}>{}:{:?}", peer, socket3.local_addr().unwrap(), packet);
    };
    let joined = future::join!(recv0, recv1, recv2, send);

    task::block_on(joined);


}
