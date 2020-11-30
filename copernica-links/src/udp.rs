use {
    crate::{Link, encode, decode},
    copernica_common::{
        InterLinkPacket, LinkId, ReplyTo, LinkPacket
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
    name: String,
    link_id: LinkId,
    l2bs_tx: Sender<InterLinkPacket>,
    bs2l_rx: Receiver<InterLinkPacket>,
}

impl UdpIp {
}

impl Link<'_> for UdpIp {
    fn new(name: String
        , link_id: LinkId
        , (l2bs_tx, bs2l_rx): ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> )
        ) -> Result<UdpIp>
    {
        trace!("LISTEN ON {:?}:", link_id);
        match link_id.reply_to()? {
            ReplyTo::UdpIp(_) => return Ok(UdpIp { name, link_id, l2bs_tx, bs2l_rx }),
            _ => return Err(anyhow!("UdpIp Link expects a LinkId of type Link.ReplyTo::UdpIp(...)")),
        }
    }

    #[allow(unreachable_code)]
    fn run(&self) -> Result<()> {
        let name = self.name.clone();
        let this_link = self.link_id.clone();
        let l2bs_tx = self.l2bs_tx.clone();
        std::thread::spawn(move || {
            task::block_on(async move {
                match this_link.reply_to()? {
                    ReplyTo::UdpIp(listen_addr) => {
                        match UdpSocket::bind(listen_addr).await {
                            Ok(socket) => {
                                loop {
                                    let mut buf = vec![0u8; 1500];
                                    match socket.recv_from(&mut buf).await {
                                        Ok((n, _peer)) => {
                                            let lp: LinkPacket = decode(buf[..n].to_vec(), this_link.sid()?)?;
                                            debug!("{} {:?}", name, this_link);
                                            let link_id = LinkId::new(this_link.lookup_id()?, this_link.sid()?, this_link.rx_pid()?, lp.reply_to());
                                            let ilp = InterLinkPacket::new(link_id, lp);
                                            let _r = l2bs_tx.send(ilp)?;
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
        let name = self.name.clone();
        let this_link = self.link_id.clone();
        let bs2l_rx = self.bs2l_rx.clone();
        std::thread::spawn(move || {
            task::block_on(async move {
                match UdpSocket::bind("127.0.0.1:0").await {
                    Ok(socket) => {
                        loop {
                            match bs2l_rx.recv(){
                                Ok(ilp) => {
                                    match ilp.reply_to()? {
                                        ReplyTo::UdpIp(remote_addr) => {
                                            let wp = ilp.link_packet().change_origination(this_link.reply_to()?);
                                            debug!("{} {:?}", name, this_link);
                                            let enc = encode(wp, this_link.sid()?, None)?;
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

