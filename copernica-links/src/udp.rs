use {
    crate::{Transport, encode, decode},
    copernica_core::{
        InterLinkPacket, Link, ReplyTo, WirePacket
    },
    anyhow::{anyhow, Result},
    crossbeam_channel::{Sender, Receiver},
    async_std::{
        net::UdpSocket,
        task,
    },
    log::{debug, error, trace},
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
        trace!("LISTEN ON {:?}:", link);
        match link.reply_to() {
            ReplyTo::UdpIp(_) => return Ok(UdpIp { link, t2c_tx, c2t_rx }),
            _ => return Err(anyhow!("UdpIp Transport expects a LinkId of type Link.ReplyTo::UdpIp(...)")),
        }
    }

    #[allow(unreachable_code)]
    fn run(&self) -> Result<()> {
        let this_link = self.link.clone();
        let t2c_tx = self.t2c_tx.clone();
        std::thread::spawn(move || {
            task::block_on(async move {
                match this_link.reply_to() {
                    ReplyTo::UdpIp(listen_addr) => {
                        match UdpSocket::bind(listen_addr).await {
                            Ok(socket) => {
                                loop {
                                    let mut buf = vec![0u8; 1500];
                                    match socket.recv_from(&mut buf).await {
                                        Ok((n, _peer)) => {
                                            let wp: WirePacket = decode(buf[..n].to_vec())?;
                                            debug!("Udp Recv on {:?} => {:?}", this_link, wp);
                                            let link = Link::new(this_link.id(), wp.reply_to());
                                            let ilp = InterLinkPacket::new(link, wp);
                                            let _r = t2c_tx.send(ilp)?;
                                        },
                                        Err(error) => error!("{:?}: {}", this_link, error),
                                    };
                                }
                            },
                            Err(error) => error!("{:?}: {}", this_link, error),
                        }
                    },
                    _ => {},
                }
                Ok::<(), anyhow::Error>(())
            })
        });
        let this_link = self.link.clone();
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
                                            let wp = ilp.wire_packet().change_origination(this_link.reply_to());
                                            debug!("Udp Send on {:?} => {:?}", this_link, wp);
                                            let enc = encode(wp)?;
                                            socket.send_to(&enc, remote_addr).await?;
                                        },
                                        _ => {},
                                    }
                                },
                                Err(error) => error!("{:?}: {}", this_link, error),
                            }
                        }
                    },
                    Err(error) => error!("{:?}: {}", this_link, error),
                }
                Ok::<(), anyhow::Error>(())
            })
        });
        Ok(())
    }
}

