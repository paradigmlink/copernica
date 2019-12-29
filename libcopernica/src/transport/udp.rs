use {
    crate::{
        transport::packet::{
            TransportPacket, TransportResponse, ReplyTo,
        },
        serdeser::{serialize, deserialize},
    },
    crossbeam_channel::{Sender, Receiver},
    async_std::{
        net::UdpSocket,
        task,
    },
};

pub fn receive_transport_packet(listen_addr: ReplyTo,
    transport_packet_sender: Sender<TransportPacket> ){
    task::block_on(async move {
        match listen_addr {
            ReplyTo::Udp(listen_addr) => {
                match UdpSocket::bind(listen_addr).await {
                    Ok(socket) => {
                        loop {
                            let mut buf = vec![0u8; 1500];
                            match socket.recv_from(&mut buf).await {
                                Ok((n, _peer)) => {
                                    match deserialize(&buf[..n]) {
                                        Ok(tp) => {
                                            let _r = transport_packet_sender.send(tp);
                                        },
                                        Err(error) => { eprintln!("{}", error) },
                                    }
                                },
                                _ => {},
                            };
                        }
                    },
                    _ => {},
                }
            },
            _ => {},
        }
    });
}

pub fn send_transport_packet(remote_addr: ReplyTo, transport_packet_receiver: Receiver<TransportPacket>) {
    task::block_on(async move {
        match UdpSocket::bind("127.0.0.1:0").await {
            Ok(socket) => {
                loop {
                    match remote_addr {
                        ReplyTo::Udp(remote_addr) => {
                            match transport_packet_receiver.recv() {
                                Ok(transport_packet) => {
                                    let transport_packet = serialize(&transport_packet);
                                    let _r = socket.send_to(&transport_packet, remote_addr).await;
                                },
                                _ => {},
                            }
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        }
    });
}

pub fn relay_transport_packet(listen_addr: ReplyTo, transport_packet_receiver: Receiver<(ReplyTo, TransportPacket)>) {
    task::block_on(async move {
        match UdpSocket::bind("127.0.0.1:0").await{
            Ok(socket) => {
                loop {
                    match transport_packet_receiver.recv(){
                        Ok((remote, transport_packet)) => {
                            match remote {
                                ReplyTo::Udp(remote_addr) => {
                                    let new_transport_packet = TransportPacket::new(listen_addr.clone(), transport_packet.payload());
                                    let new_transport_packet = serialize(&new_transport_packet);
                                    let _r = socket.send_to(&new_transport_packet, remote_addr).await;
                                },
                                _ => {},
                            }
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        }

    });
}

pub fn send_transport_response(listen_addr: ReplyTo, transport_response_receiver: Receiver<TransportResponse>) {
    task::block_on(async move {
        match UdpSocket::bind("127.0.0.1:0").await {
            Ok(socket) => {
                loop {
                    match transport_response_receiver.recv() {
                        Ok(tr) => {
                            let reply_to: ReplyTo = tr.reply_to();
                            match reply_to {
                                ReplyTo::Udp(remote_addr) => {
                                    for (_seq, narrow_waist_packet) in tr.payload().iter() {
                                        let transport_packet = TransportPacket::new(listen_addr.clone(), narrow_waist_packet.clone());
                                        let transport_packet = serialize(&transport_packet);
                                        let _r = socket.send_to(&transport_packet, remote_addr).await;
                                    }
                                },
                                _ => {},
                            }
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        }
    });
}
