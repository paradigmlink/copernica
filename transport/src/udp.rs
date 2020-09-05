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
    t2c_tx: Sender<InterLinkPacket>,
    c2t_rx: Receiver<InterLinkPacket>,
}

impl UdpIp {
}

impl Transport<'_> for UdpIp {
    fn new(link: Link
        , (t2c_tx, c2t_rx): ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> )
        ) -> Result<UdpIp>
    {
        match link.reply_to() {
            ReplyTo::UdpIp(_) => return Ok(UdpIp { link, t2c_tx, c2t_rx }),
            _ => return Err(anyhow!("UdpIp Transport expects a LinkId of type Link.ReplyTo::UdpIp(...)")),
        }
    }

    #[allow(unreachable_code)]
    fn run(&self) -> Result<()> {
        let link = self.link.clone();
        let t2c_tx = self.t2c_tx.clone();
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
                                            // https://docs.rs/reed-solomon/0.2.1/reed_solomon/
                                            let wp = WirePacket::try_from_slice(&buf[..n])?;
                                            let ilp = InterLinkPacket::new(link.clone(), wp);
                                            let _r = t2c_tx.send(ilp)?;
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
        let c2t_rx = self.c2t_rx.clone();
        std::thread::spawn(move || {
            task::block_on(async move {
                match UdpSocket::bind("127.0.0.1:0").await {
                    Ok(socket) => {
                        loop {
                            match c2t_rx.recv(){
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

