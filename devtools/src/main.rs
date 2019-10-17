extern crate packets;

use async_std::io;
use async_std::net::UdpSocket;
use async_std::task;
use packets::{Packet, mk_interest};
use bincode::{serialize, deserialize};

fn main() -> io::Result<()> {
    task::block_on(async {
        let socket = UdpSocket::bind("127.0.0.1:8081").await?;
        println!("Listening on {}", socket.local_addr()?);

        let msg = mk_interest("hello world".to_string());
        println!("To Send : {:?}", &msg);
        let msg = serialize(&msg).unwrap();
        println!("<- {:?}", msg);
        socket.send_to(&msg, "127.0.0.1:8090").await?;
        socket.send_to(&msg, "127.0.0.1:8091").await?;
        socket.send_to(&msg, "127.0.0.1:8092").await?;


        //let mut buf = vec![0u8; 1024];
        //let (n, _) = socket.recv_from(&mut buf).await?;
        //println!("-> {:?}", &buf[..n]);
        //let packet: Packet = deserialize(&buf[..n]).unwrap();
        //println!("Returned: {:?}", packet);

        Ok(())
    })
}
