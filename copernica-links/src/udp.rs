use {
    crate::{Link, encode, decode},
    copernica_common::{ InterLinkPacket, LinkId, ReplyTo, Operations },
    anyhow::{anyhow, Result},
    crossbeam_channel::{Receiver, Sender},
    futures_lite::{future},
    log::{error, trace},
    std::{
      net::{SocketAddr, UdpSocket},
      sync::{Arc, Mutex},
    },
};
#[allow(dead_code)]
pub struct UdpIp {
    label: String,
    link_id: LinkId,
    ops: Operations,
    l2bs_tx: Sender<InterLinkPacket>,
    bs2l_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
}
impl Link for UdpIp {
    fn new(link_id: LinkId
        , (label, ops): (String, Operations)
        , (l2bs_tx, bs2l_rx): ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> )
        ) -> Result<UdpIp>
    {
        trace!("LISTEN ON {:?}:", link_id);
        ops.register_link(label.clone());
        match link_id.reply_to()? {
            ReplyTo::UdpIp(_) => return Ok(UdpIp { label, link_id, ops, l2bs_tx, bs2l_rx: Arc::new(Mutex::new(bs2l_rx)) }),
            _ => return Err(anyhow!("UdpIp Link expects a LinkId of type Link.ReplyTo::UdpIp(...)")),
        }
    }
    #[allow(unreachable_code)]
    fn run(&mut self) -> Result<()> {
        let this_link = self.link_id.clone();
        let l2bs_tx = self.l2bs_tx.clone();
        let ops = self.ops.clone();
        let label = self.label.clone();
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
                                        match decode(buf[..n].to_vec(), this_link.clone()) {
                                            Ok((_lnk_tx_pid, lp)) => {
                                                trace!("\t\t\t|  |  link-to-broker-or-protocol");
                                                trace!("\t\t\t|  |  {}", this_link.lookup_id()?);
                                                ops.message_from(label.clone());
                                                let link_id = LinkId::new(this_link.lookup_id()?, this_link.link_sid()?, this_link.remote_link_pid()?, lp.reply_to());
                                                let ilp = InterLinkPacket::new(link_id, lp);
                                                match l2bs_tx.send(ilp) {
                                                    Ok(_) => {},
                                                    Err(e) => error!("udp_ip link {:?}", e),
                                                }
                                            },
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
        let bs2l_rx = self.bs2l_rx.clone();
        let ops = self.ops.clone();
        let label = self.label.clone();
        std::thread::spawn(move || {
            match async_io::Async::<UdpSocket>::bind(SocketAddr::new("127.0.0.1".parse()?, 0)) {
                Ok(socket) => {
                    let bs2l_rx = bs2l_rx.lock().unwrap();
                    loop {
                        match bs2l_rx.recv() {
                            Ok(ilp) => {
                                match ilp.reply_to()? {
                                    ReplyTo::UdpIp(remote_addr) => {
                                        let lp = ilp.link_packet().change_origination(this_link.reply_to()?);
                                        trace!("\t\t\t|  |  broker-or-protocol-to-link");
                                        trace!("\t\t\t|  |  {}", this_link.lookup_id()?);
                                        ops.message_from(label.clone());
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

