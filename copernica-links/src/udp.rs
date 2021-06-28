use {
    crate::{Link, encode, decode},
    copernica_common::{ InterLinkPacket, LinkId, ReplyTo },
    copernica_monitor::{LogEntry},
    anyhow::{anyhow, Result},
    std::sync::mpsc::{Receiver, SyncSender},
    futures_lite::{future},
    log::{debug, error, trace},
    std::{
      net::{SocketAddr, UdpSocket},
    },
};
pub struct UdpIp {
    link_id: LinkId,
    l2bs_tx: SyncSender<InterLinkPacket>,
    bs2l_rx: Receiver<InterLinkPacket>,
}
impl Link<'_> for UdpIp {
    fn new(link_id: LinkId
        , label: &str
        , (l2bs_tx, bs2l_rx): ( SyncSender<InterLinkPacket> , Receiver<InterLinkPacket> )
        ) -> Result<UdpIp>
    {
        trace!("LISTEN ON {:?}:", link_id);
        debug!("{}", LogEntry::link(link_id.link_pid()?, label));
        match link_id.reply_to()? {
            ReplyTo::UdpIp(_) => return Ok(UdpIp { link_id, l2bs_tx, bs2l_rx }),
            _ => return Err(anyhow!("UdpIp Link expects a LinkId of type Link.ReplyTo::UdpIp(...)")),
        }
    }
    #[allow(unreachable_code)]
    fn run(self) -> Result<()> {
        let this_link = self.link_id.clone();
        let l2bs_tx = self.l2bs_tx.clone();
        std::thread::spawn(move || {
            match this_link.reply_to()? {
                ReplyTo::UdpIp(addr) => {
                    match async_io::Async::<UdpSocket>::bind(addr) {
                        Ok(socket) => {
                            loop {
                                let mut buf = vec![0u8; 1500];
                                let data = future::block_on(async{ socket.recv_from(&mut buf).await });
                                match data {
                                    Ok((n, _peer)) => {
                                        debug!("\t\t\t|  |  link-to-broker-or-protocol");
                                        trace!("\t\t\t|  |  {}", this_link.lookup_id()?);
                                        let (_lnk_tx_pid, lp) = decode(buf[..n].to_vec(), this_link.clone())?;
                                        let link_id = LinkId::new(this_link.lookup_id()?, this_link.link_sid()?, this_link.remote_link_pid()?, lp.reply_to());
                                        let ilp = InterLinkPacket::new(link_id, lp);
                                        match l2bs_tx.send(ilp) {
                                            Ok(_) => {},
                                            Err(e) => error!("udp_ip link {:?}", e),
                                        }
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
        });
        let this_link = self.link_id.clone();
        let bs2l_rx = self.bs2l_rx;
        std::thread::spawn(move || {
            match async_io::Async::<UdpSocket>::bind(SocketAddr::new("127.0.0.1".parse()?, 0)) {
                Ok(socket) => {
                    loop {
                        match bs2l_rx.recv() {
                            Ok(ilp) => {
                                match ilp.reply_to()? {
                                    ReplyTo::UdpIp(remote_addr) => {
                                        let lp = ilp.link_packet().change_origination(this_link.reply_to()?);
                                        debug!("\t\t\t|  |  broker-or-protocol-to-link");
                                        trace!("\t\t\t|  |  {}", this_link.lookup_id()?);
                                        let enc = encode(lp, this_link.clone())?;
                                        let data = future::block_on(async{ socket.send_to(&enc, remote_addr).await });
                                        match data {
                                            Ok(_) => {},
                                            Err(_e) => {},
                                        }
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
        });
        Ok(())
    }
}

