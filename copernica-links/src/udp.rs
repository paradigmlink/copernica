use {
    crate::{Link, encode, decode},
    copernica_common::{
        InterLinkPacket, LinkId, ReplyTo
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
    link_id: LinkId,
    l2bs_tx: Sender<InterLinkPacket>,
    bs2l_rx: Receiver<InterLinkPacket>,
}

impl UdpIp {
}

impl Link<'_> for UdpIp {
    fn new(link_id: LinkId
        , (l2bs_tx, bs2l_rx): ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> )
        ) -> Result<UdpIp>
    {
        trace!("LISTEN ON {:?}:", link_id);
        match link_id.reply_to()? {
            ReplyTo::UdpIp(_) => return Ok(UdpIp { link_id, l2bs_tx, bs2l_rx }),
            _ => return Err(anyhow!("UdpIp Link expects a LinkId of type Link.ReplyTo::UdpIp(...)")),
        }
    }

    #[allow(unreachable_code)]
    fn run(&self) -> Result<()> {
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
                                            debug!("\t\t\t|  |  link-to-broker-or-protocol");
                                            trace!("\t\t\t|  |  {}", this_link.lookup_id()?);
                                            let (_lnk_tx_pid, lp) = decode(buf[..n].to_vec(), this_link.clone())?;
                                            let link_id = LinkId::new(this_link.lookup_id()?, this_link.link_sid()?, this_link.remote_link_pid()?, lp.reply_to());
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
                                            let lp = ilp.link_packet().change_origination(this_link.reply_to()?);
                                            debug!("\t\t\t|  |  broker-or-protocol-to-link");
                                            trace!("\t\t\t|  |  {}", this_link.lookup_id()?);
                                            let enc = encode(lp, this_link.clone())?;
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

