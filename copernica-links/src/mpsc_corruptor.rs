use {
    crate::{Link, decode, encode},
    copernica_common::{
        InterLinkPacket, LinkId, ReplyTo, constants, Operations
    },
    anyhow::{anyhow, Result},
    crossbeam_channel::{Receiver, Sender, bounded},
    log::{trace, error, debug},
};
#[derive(Clone)]
pub enum Corruption {
    Immune,
    Integrity,
    Order,
    Presence,
}
#[allow(dead_code)]
pub struct MpscCorruptor {
    label: String,
    corruption: Corruption,
    link_id: LinkId,
    ops: Operations,
    // t = tansport; c = copernic; 0 = this instance of t; 1 = the pair of same type
    l2bs_tx: Sender<InterLinkPacket>,
    bs2l_rx: Receiver<InterLinkPacket>,
    l2l0_tx: Sender<Vec<u8>>,
    l2l0_rx: Receiver<Vec<u8>>,
    l2l1_tx: Option<Vec<Sender<Vec<u8>>>>,
}
impl MpscCorruptor {
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
    pub fn corrupt(&mut self, corruption: Corruption) {
        self.corruption = corruption;
    }
}

impl Link for MpscCorruptor {
    fn new(link_id: LinkId
        , (label, ops): (String, Operations)
        , (l2bs_tx, bs2l_rx): ( Sender<InterLinkPacket>, Receiver<InterLinkPacket> )
        ) -> Result<MpscCorruptor> {
        ops.register_link(label.clone());
        match link_id.reply_to()? {
            ReplyTo::Mpsc => {
                let (l2l0_tx, l2l0_rx) = bounded::<Vec<u8>>(constants::BOUNDED_BUFFER_SIZE);
                return Ok(
                    MpscCorruptor {
                        label,
                        link_id,
                        ops,
                        corruption: Corruption::Immune,
                        l2bs_tx,
                        bs2l_rx,
                        l2l0_tx,
                        l2l0_rx,
                        l2l1_tx: None,
                    }
                )
            }
            _ => return Err(anyhow!("MpscCorruptor Link expects a LinkId of type LinkId::Mpsc")),
        }
    }
    #[allow(unreachable_code)]
    fn run(&mut self) -> Result<()> {
        let this_link = self.link_id.clone();
        trace!("Started {:?}:", this_link);
        let l2l0_rx = self.l2l0_rx.clone();
        let l2bs_tx = self.l2bs_tx.clone();
        let ops = self.ops.clone();
        let label = self.label.clone();
        std::thread::spawn(move || {
            match this_link.reply_to()? {
                ReplyTo::Mpsc => {
                    loop {
                        match l2l0_rx.recv() {
                            Ok(msg) => {
                                match decode(msg, this_link.clone()) {
                                    Ok((_lnk_tx_pid, lp)) => {
                                        let link_id = LinkId::new(this_link.lookup_id()?, this_link.link_sid()?, this_link.remote_link_pid()?, lp.reply_to());
                                        let ilp = InterLinkPacket::new(link_id, lp.clone());
                                        trace!("\t|  |  link-to-broker-or-protocol");
                                        trace!("\t|  |  {}", this_link.lookup_id()?);
                                        ops.message_from(label.clone());
                                        match l2bs_tx.send(ilp) {
                                            Ok(_) => {},
                                            Err(e) => error!("mpsc_corruptor {:?}", e),
                                        }
                                    },
                                    Err(e) => error!("{:?}", e),
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
        let bs2l_rx = self.bs2l_rx.clone();
        let ops = self.ops.clone();
        let label = self.label.clone();
        let corruption = self.corruption.clone();
        if let Some(l2l1_tx) = self.l2l1_tx.clone() {
            std::thread::spawn(move || {
                let mut has_order_skipped_first = false;
                let mut has_presence_skipped_first = false;
                let mut has_integrity_skipped_first = false;
                let mut has_order_been_corrupted = false;
                let mut has_memory_been_sent = false;
                let mut has_presence_been_corrupted = false;
                let mut has_integrity_been_corrupted = false;
                let mut memory: Vec<u8> = vec![];
                loop {
                    match bs2l_rx.recv() {
                        Ok(ilp) => {
                            let lp = ilp.link_packet().change_origination(this_link.reply_to()?);
                            let enc = encode(lp.clone(), this_link.clone())?;
                            let mut corrupted = enc;
                            match corruption {
                                Corruption::Immune => {
                                    //debug!("Immune");
                                    for s in l2l1_tx.clone() {
                                        ops.message_from(label.clone());
                                        s.send(corrupted.clone())?;
                                    }
                                },
                                Corruption::Integrity => {
                                    if has_integrity_skipped_first {
                                        if has_integrity_been_corrupted {
                                            for s in l2l1_tx.clone() {
                                                ops.message_from(label.clone());
                                                s.send(corrupted.clone())?;
                                            }
                                        } else {
                                            has_integrity_been_corrupted = true;
                                            debug!("Corrupt Integrity");
                                            for i in 4..8 {
                                                corrupted[i] = 0x0;
                                            }
                                            for s in l2l1_tx.clone() {
                                                ops.message_from(label.clone());
                                                s.send(corrupted.clone())?;
                                            }
                                        }
                                    } else {
                                        has_integrity_skipped_first = true;
                                        for s in l2l1_tx.clone() {
                                            ops.message_from(label.clone());
                                            s.send(corrupted.clone())?;
                                        }
                                    }
                                },
                                Corruption::Order => {
                                    if has_order_skipped_first {
                                        if has_order_been_corrupted {
                                            if has_memory_been_sent {
                                                for s in l2l1_tx.clone() {
                                                    ops.message_from(label.clone());
                                                    s.send(corrupted.clone())?;
                                                }
                                            } else {
                                                has_memory_been_sent = true;
                                                for s in l2l1_tx.clone() {
                                                    ops.message_from(label.clone());
                                                    s.send(memory.clone())?;
                                                }
                                                for s in l2l1_tx.clone() {
                                                    ops.message_from(label.clone());
                                                    s.send(corrupted.clone())?;
                                                }
                                            }
                                        } else {
                                            memory = corrupted;
                                            has_order_been_corrupted = true;
                                            debug!("Corrupt Order");
                                            continue
                                        }
                                    } else {
                                        has_order_skipped_first = true;
                                        for s in l2l1_tx.clone() {
                                            ops.message_from(label.clone());
                                            s.send(corrupted.clone())?;
                                        }
                                    }
                                },
                                Corruption::Presence => {
                                    if has_presence_skipped_first {
                                        if has_presence_been_corrupted {
                                            for s in l2l1_tx.clone() {
                                                ops.message_from(label.clone());
                                                s.send(corrupted.clone())?;
                                            }
                                        } else {
                                            has_presence_been_corrupted = true;
                                            debug!("Corrupt Presence");
                                            continue
                                        }
                                    } else {
                                        has_presence_skipped_first = true;
                                        for s in l2l1_tx.clone() {
                                            ops.message_from(label.clone());
                                            s.send(corrupted.clone())?;
                                        }
                                    }
                                },
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

