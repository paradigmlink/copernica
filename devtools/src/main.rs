extern crate packets;

use async_std::io;
use async_std::net::UdpSocket;
use async_std::task;
use packets::{Packet, mk_interest};
use bincode::{serialize, deserialize};
use std::{thread, time};

fn main() -> io::Result<()> {
    task::block_on(async {
        let socket = UdpSocket::bind("127.0.0.1:8081").await?;
        println!("Listening on {}", socket.local_addr()?);

        let msg0 = mk_interest("hello".to_string());
        println!("To Send : {:?}", &msg0);
        let msg0 = serialize(&msg0).unwrap();
        let msg1 = mk_interest("world".to_string());
        println!("To Send : {:?}", &msg1);
        let msg1 = serialize(&msg1).unwrap();
        let msg2 = mk_interest("hello world".to_string());
        println!("To Send : {:?}", &msg2);
        let msg2 = serialize(&msg2).unwrap();
        socket.send_to(&msg0, "127.0.0.1:8090").await?;
        thread::sleep(time::Duration::from_millis(1));
        socket.send_to(&msg1, "127.0.0.1:8091").await?;
        thread::sleep(time::Duration::from_millis(1));
        socket.send_to(&msg2, "127.0.0.1:8092").await?;

        Ok(())
    })
}
