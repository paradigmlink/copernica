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

        println!("Listening on {}", socket.local_addr().unwrap());

        let msg0 = mk_interest("oa".to_string());
        //let msg0 = mk_interest("h/el/loo/uh/tn/oh/unt/oe/uho/nt/eu/han/to/uha/on/tuh/ne/otu/hao/nt/uh/aon/tu/hoa/nt/uho/an/tuh/oen/tu/ho/au/nt/eoh/un/toa/eh/un/eot/hu/oan/et/uhe/oa/ntu/ha/eon/tu/he/oa".to_string());
        println!("To Send : {:?}", &msg0);
        let msg0 = serialize(&msg0).unwrap();
        /*let msg1 = mk_interest("world".to_string());
        println!("To Send : {:?}", &msg1);
        let msg1 = serialize(&msg1).unwrap();
        let msg2 = mk_interest("hello world".to_string());
        println!("To Send : {:?}", &msg2);
        let msg2 = serialize(&msg2).unwrap();
        */
        socket.send_to(&msg0, "127.0.0.1:8092").await.unwrap();
        /*
        thread::sleep(time::Duration::from_millis(3));
        socket.send_to(&msg1, "127.0.0.1:8091").await?;
        thread::sleep(time::Duration::from_millis(1));
        socket.send_to(&msg2, "127.0.0.1:8092").await?;
*/
    };
    let recv = async {
        let socket1 = UdpSocket::bind("127.0.0.1:8093").await.unwrap();
        let mut buf = vec![0u8; 1024];
        println!("Listening on {}", socket1.local_addr().unwrap());
        let (n, peer) = socket1.recv_from(&mut buf).await.unwrap();
        println!("Peer: {:?}", peer);
        let packet: Packet = deserialize(&buf[..n]).unwrap();
        println!("Received: {:?}", packet);

    };

    let joined = future::join!(recv, send);

    task::block_on(joined);


}
