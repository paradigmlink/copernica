use {
    crate::{Transport},
    copernica::{
        TransportPacket, ReplyTo, LinkId
    },
    borsh::{BorshDeserialize, BorshSerialize},
    anyhow::{anyhow, Result},
    crossbeam_channel::{Sender, Receiver},
    async_std::{
        net::UdpSocket,
        task,
    },
};

pub struct UdpIp {
    link_id: LinkId,
    router_inbound_tx: Sender<(LinkId, TransportPacket)>,
    router_outbound_rx: Receiver<(LinkId, TransportPacket)>,
}

impl UdpIp {
}

impl Transport<'_> for UdpIp {
    fn new(link_id: LinkId
        , (router_inbound_tx, router_outbound_rx): ( Sender<(LinkId, TransportPacket)> , Receiver<(LinkId, TransportPacket)> )
        ) -> Result<UdpIp>
    {
        match link_id.reply_to() {
            ReplyTo::UdpIp(_) => return Ok(UdpIp { link_id, router_inbound_tx, router_outbound_rx }),
            _ => return Err(anyhow!("UdpIp Transport expects a LinkId of type ReplyTo::UdpIp(...)")),
        }
    }

    #[allow(unreachable_code)]
    fn run(&self) -> Result<()> {
        let link_id = self.link_id.clone();
        let router_inbound_tx = self.router_inbound_tx.clone();
        std::thread::spawn(move || {
            task::block_on(async move {
                match link_id.reply_to() {
                    ReplyTo::UdpIp(listen_addr) => {
                        match UdpSocket::bind(listen_addr).await {
                            Ok(socket) => {
                                loop {
                                    let mut buf = vec![0u8; 1500];
                                    match socket.recv_from(&mut buf).await {
                                        Ok((n, _peer)) => {
                                            let tp = TransportPacket::try_from_slice(&buf[..n])?;
                                            let li = LinkId::new(tp.clone().reply_to(), link_id.nonce());
                                            let _r = router_inbound_tx.send((li, tp))?;
                                        },
                                        Err(error) => return Err(anyhow!("{}", error)),
                                    };
                                }
                            },
                            Err(error) => return Err(anyhow!("{}", error)),
                        }
                    },
                    _ => {},
                }
                Ok::<(), anyhow::Error>(())
            })
        });
        let link_id = self.link_id.clone();
        let router_outbound_rx = self.router_outbound_rx.clone();
        std::thread::spawn(move || {
            task::block_on(async move {
                match UdpSocket::bind("127.0.0.1:0").await {
                    Ok(socket) => {
                        loop {
                            match router_outbound_rx.recv(){
                                Ok((msg_link_id, transport_packet)) => {
                                    let rt = msg_link_id.reply_to();
                                    match rt {
                                        ReplyTo::UdpIp(remote_addr) => {
                                            let transport_packet = TransportPacket::new(link_id.clone().reply_to(), transport_packet.payload());
                                            let transport_packet: Vec<u8> = transport_packet.try_to_vec()?;
                                            socket.send_to(&transport_packet, remote_addr).await?;
                                        },
                                        _ => {},
                                    }
                                },
                                Err(error) => return Err(anyhow!("{}", error)),
                            }
                        }
                    },
                    Err(error) => return Err(anyhow!("{}", error)),
                }
                Ok::<(), anyhow::Error>(())
            })
        });
        Ok(())
    }
}

