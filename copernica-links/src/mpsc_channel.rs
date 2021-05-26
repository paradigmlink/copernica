use {
    crate::{Link, decode, encode},
    copernica_common::{
        InterLinkPacket, LinkId, ReplyTo, constants
    },
    anyhow::{anyhow, Result},
    futures_lite::{future},
    async_executor::{Executor},
    futures::{
        stream::{self, StreamExt},
        channel::mpsc::{Sender, Receiver, channel},
        sink::{SinkExt},
    },
    log::{debug, trace, error },
    std::sync::{Arc, Mutex},
};

pub struct MpscChannel {
    link_id: LinkId,
    // t = tansport; c = copernic; 0 = this instance of t; 1 = the pair of same type
    l2bs_tx: Sender<InterLinkPacket>,
    bs2l_rx: Receiver<InterLinkPacket>,
    l2l0_tx: Sender<Vec<u8>>,        // give
    l2l0_rx: Receiver<Vec<u8>>,      // keep
    l2l1_tx: Option<Vec<Sender<Vec<u8>>>>,
}

impl MpscChannel {
    pub fn male(&self) -> Sender<Vec<u8>> {
        self.l2l0_tx.clone()
    }
    pub fn female(&mut self, new_l2l1_tx: Sender<Vec<u8>>) {
        if let None = self.l2l1_tx {
            self.l2l1_tx = Some(vec![]);
        }
        if let Some(l2l1_tx) = &mut self.l2l1_tx {
            l2l1_tx.push(new_l2l1_tx);
        }
    }
}

impl<'a> Link<'a> for MpscChannel {
    fn new(link_id: LinkId
        , (l2bs_tx, bs2l_rx): ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> )
        ) -> Result<MpscChannel> {
        match link_id.reply_to()? {
            ReplyTo::Mpsc => {
                let (l2l0_tx, l2l0_rx) = channel::<Vec<u8>>(constants::BOUNDED_BUFFER_SIZE);
                return Ok(
                    MpscChannel {
                        link_id,
                        l2bs_tx,
                        bs2l_rx,
                        l2l0_tx,
                        l2l0_rx,
                        l2l1_tx: None,
                    })
            }
            _ => return Err(anyhow!("MpscChannel Link expects a LinkId of type LinkId::Mpsc")),
        }
    }

    #[allow(unreachable_code)]
    fn run(self) -> Result<()> {
        let this_link = self.link_id.clone();
        trace!("Started {:?}:", this_link);
        let mut l2l0_rx = self.l2l0_rx;
        let mut l2bs_tx = self.l2bs_tx;
        let ex = Executor::new();
        let task = ex.spawn(async move {
            match this_link.reply_to()? {
                ReplyTo::Mpsc => {
                    loop {
                        match l2l0_rx.next().await {
                            Some(msg) => {
                                let (_lnk_tx_pid, lp) = decode(msg, this_link.clone())?;
                                let link_id = LinkId::new(this_link.lookup_id()?, this_link.link_sid()?, this_link.remote_link_pid()?, lp.reply_to());
                                let ilp = InterLinkPacket::new(link_id, lp.clone());
                                debug!("\t\t|  |  link-to-broker-or-protocol");
                                trace!("\t|  |  {}", this_link.lookup_id()?);
                                match l2bs_tx.send(ilp).await {
                                    Ok(_) => {},
                                    Err(e) => error!("mpsc_channel {:?}", e),
                                }
                            },
                            None => {}
                        };
                    }
                },
                _ => {},
            }
            Ok::<(), anyhow::Error>(())
        });
        std::thread::spawn(move || future::block_on(ex.run(task)));
        let this_link = self.link_id.clone();
        let mut bs2l_rx = self.bs2l_rx;
        if let Some(l2l1_tx) = self.l2l1_tx.clone() {
            let ex = Executor::new();
            let task = ex.spawn(async move {
                loop {
                    match bs2l_rx.next().await {
                        Some(ilp) => {
                            let lp = ilp.link_packet().change_origination(this_link.reply_to()?);
                            let enc = encode(lp, this_link.clone())?;
                            for mut s in l2l1_tx.clone() {
                                debug!("\t\t|  |  broker-or-protocol-to-link");
                                trace!("\t\t|  |  {}", this_link.lookup_id()?);
                                match s.send(enc.clone()).await {
                                    Ok(_) => {},
                                    Err(e) => error!("mpsc_channel outbound: {:?}", e),
                                }
                            }
                        },
                        None => {}
                    }
                }
                Ok::<(), anyhow::Error>(())
            });
            std::thread::spawn(move || future::block_on(ex.run(task)));
        } else {
            return Err(anyhow!("You need to bind the transports before using them, i.e. t0.female(t1.male()); followed by: t1.female(t0.male());"))
        }

        Ok(())
    }
}

