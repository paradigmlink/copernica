use {
    crate::{Link, decode, encode},
    copernica_common::{
        InterLinkPacket, LinkId, ReplyTo, constants
    },
    copernica_monitor::{LogEntry},
    anyhow::{anyhow, Result},
    std::sync::mpsc::{Receiver, SyncSender, sync_channel as channel},
    log::{debug, trace, error},
};
pub struct MpscCorruptor {
    link_id: LinkId,
    // t = tansport; c = copernic; 0 = this instance of t; 1 = the pair of same type
    t2c_tx: SyncSender<InterLinkPacket>,
    c2t_rx: Receiver<InterLinkPacket>,
    t2t0_tx: SyncSender<Vec<u8>>,        // give
    t2t0_rx: Receiver<Vec<u8>>,      // keep
    t2t1_tx: Option<Vec<SyncSender<Vec<u8>>>>,
}

impl MpscCorruptor {
    pub fn male(&self) -> SyncSender<Vec<u8>> {
        self.t2t0_tx.clone()
    }
    pub fn female(&mut self, new_t2t1_tx: SyncSender<Vec<u8>>) {
        if let None = self.t2t1_tx {
            self.t2t1_tx = Some(vec![]);
        }
        if let Some(t2t1_tx) = &mut self.t2t1_tx {
            t2t1_tx.push(new_t2t1_tx);
        }
    }
}

impl<'a> Link<'a> for MpscCorruptor {
    fn new(link_id: LinkId
        , label: &str
        , (t2c_tx, c2t_rx): ( SyncSender<InterLinkPacket> , Receiver<InterLinkPacket> )
        ) -> Result<MpscCorruptor> {
        debug!("{}", LogEntry::link(link_id.link_pid()?, label));
        match link_id.reply_to()? {
            ReplyTo::Mpsc => {
                let (t2t0_tx, t2t0_rx) = channel::<Vec<u8>>(constants::BOUNDED_BUFFER_SIZE);
                return Ok(
                    MpscCorruptor {
                        link_id,
                        t2c_tx,
                        c2t_rx,
                        t2t0_tx,
                        t2t0_rx,
                        t2t1_tx: None,
                    })
            }
            _ => return Err(anyhow!("MpscCorruptor Link expects a LinkId of type LinkId::Mpsc")),
        }
    }
    #[allow(unreachable_code)]
    fn run(self) -> Result<()> {
        let this_link = self.link_id.clone();
        trace!("Started {:?}:", this_link);
        let t2t0_rx = self.t2t0_rx;
        let t2c_tx = self.t2c_tx;
        std::thread::spawn(move || {
            match this_link.reply_to()? {
                ReplyTo::Mpsc => {
                    loop {
                        match t2t0_rx.recv() {
                            Ok(msg) => {
                                let (_lnk_tx_pid, lp) = decode(msg, this_link.clone())?;
                                let link_id = LinkId::new(this_link.lookup_id()?, this_link.link_sid()?, this_link.remote_link_pid()?, lp.reply_to());
                                let ilp = InterLinkPacket::new(link_id, lp);
                                debug!("\t|  |  link-to-broker-or-protocol");
                                trace!("\t|  |  {}", this_link.lookup_id()?);
                                match t2c_tx.send(ilp) {
                                    Ok(_) => {},
                                    Err(e) => error!("mpsc_corruptor {:?}", e),
                                }
                            },
                            Err(error) => error!("{:?}: {}", this_link, error),
                        };
                    }
                },
                _ => {},
            }
            Ok::<(), anyhow::Error>(())
        });
        let this_link = self.link_id.clone();
        let c2t_rx = self.c2t_rx;
        if let Some(t2t1_tx) = self.t2t1_tx.clone() {
            std::thread::spawn(move || {
                loop {
                    match c2t_rx.recv() {
                        Ok(ilp) => {
                            let lp = ilp.link_packet().change_origination(this_link.reply_to()?);
                            let enc = encode(lp, this_link.clone())?;
                            let mut corrupted = enc;
                            for i in 4..7 {
                                corrupted[i] = 0x0;
                            }
                            for s in t2t1_tx.clone() {
                                debug!("\t|  |  broker-or-protocol-to-link");
                                trace!("\t|  |  {}", this_link.lookup_id()?);
                                match s.send(corrupted.clone()) {
                                    Ok(_) => {},
                                    Err(e) => error!("mpsc_corruptor {:?}", e),
                                }
                            }
                        },
                        Err(error) => error!("{:?}: {}", this_link, error),
                    }
                }
                Ok::<(), anyhow::Error>(())
            });
        } else {
            return Err(anyhow!("You need to bind the transports before using them, i.e. t0.female(t1.male()); followed by: t1.female(t0.male());"))
        }

        Ok(())
    }
}

