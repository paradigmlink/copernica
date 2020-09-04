use {
    crate::{Transport},
    copernica::{
        InterLinkPacket, WirePacket, Link, ReplyTo
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
    link: Link,
    router_inbound_tx: Sender<InterLinkPacket>,
    router_outbound_rx: Receiver<InterLinkPacket>,
}

impl UdpIp {
}

impl Transport<'_> for UdpIp {
    fn new(link: Link
        , (router_inbound_tx, router_outbound_rx): ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> )
        ) -> Result<UdpIp>
    {
        match link.reply_to() {
            ReplyTo::UdpIp(_) => return Ok(UdpIp { link, router_inbound_tx, router_outbound_rx }),
            _ => return Err(anyhow!("UdpIp Transport expects a LinkId of type Link.ReplyTo::UdpIp(...)")),
        }
    }

    #[allow(unreachable_code)]
    fn run(&self) -> Result<()> {
        let link = self.link.clone();
        let router_inbound_tx = self.router_inbound_tx.clone();
        std::thread::spawn(move || {
            task::block_on(async move {
                match link.reply_to() {
                    ReplyTo::UdpIp(listen_addr) => {
                        match UdpSocket::bind(listen_addr).await {
                            Ok(socket) => {
                                loop {
                                    let mut buf = vec![0u8; 1500];
                                    match socket.recv_from(&mut buf).await {
                                        Ok((n, _peer)) => {
                                            let wp = WirePacket::try_from_slice(&buf[..n])?;
                                            let ilp = InterLinkPacket::new(link.clone(), wp);
                                            let _r = router_inbound_tx.send(ilp)?;
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
        let link = self.link.clone();
        let router_outbound_rx = self.router_outbound_rx.clone();
        std::thread::spawn(move || {
            task::block_on(async move {
                match UdpSocket::bind("127.0.0.1:0").await {
                    Ok(socket) => {
                        loop {
                            match router_outbound_rx.recv(){
                                Ok(ilp) => {
                                    match ilp.reply_to() {
                                        ReplyTo::UdpIp(remote_addr) => {
                                            let wire_packet = WirePacket::new(link.reply_to(), ilp.narrow_waist());
                                            let wire_packet: Vec<u8> = wire_packet.try_to_vec()?;
                                            socket.send_to(&wire_packet, remote_addr).await?;
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

