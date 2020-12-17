use {
    crate::{Link, decode, encode},
    copernica_common::{
        InterLinkPacket, LinkId, ReplyTo
    },
    anyhow::{anyhow, Result},
    crossbeam_channel::{Sender, Receiver, unbounded},
    log::{debug, error, trace},
};

pub struct MpscChannel {
    name: String,
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
    fn new(name: String
        , link_id: LinkId
        , (l2bs_tx, bs2l_rx): ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> )
        ) -> Result<MpscChannel> {
        match link_id.reply_to()? {
            ReplyTo::Mpsc => {
                let (l2l0_tx, l2l0_rx) = unbounded::<Vec<u8>>();
                return Ok(
                    MpscChannel {
                        name,
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
    fn run(&self) -> Result<()> {
        let name = self.name.clone();
        let this_link = self.link_id.clone();
        trace!("Started {:?}:", this_link);
        let l2l0_rx = self.l2l0_rx.clone();
        let l2bs_tx = self.l2bs_tx.clone();
        std::thread::spawn(move || {
            match this_link.reply_to()? {
                ReplyTo::Mpsc => {
                    loop {
                        match l2l0_rx.recv(){
                            Ok(msg) => {
                                let (_lnk_tx_pid, lp) = decode(msg, this_link.clone())?;
                                let link_id = LinkId::new(this_link.lookup_id()?, this_link.sid()?, this_link.rx_pid()?, lp.reply_to());
                                let ilp = InterLinkPacket::new(link_id, lp.clone());
                                trace!("\t|  |  {}:{}", name, this_link.lookup_id()?);
                                let _r = l2bs_tx.send(ilp)?;
                            },
                            Err(error) => error!("{:?}: {}", this_link, error),
                        };
                    }
                },
                _ => {},
            }
            Ok::<(), anyhow::Error>(())
        });
        let name = self.name.clone();
        let this_link = self.link_id.clone();
        let bs2l_rx = self.bs2l_rx.clone();
        if let Some(l2l1_tx) = self.l2l1_tx.clone() {
            std::thread::spawn(move || {
                loop {
                    match bs2l_rx.recv(){
                        Ok(ilp) => {
                            let lp = ilp.link_packet().change_origination(this_link.reply_to()?);
                            let enc = encode(lp, this_link.clone())?;
                            for s in l2l1_tx.clone() {
                                debug!("\t\t|  |  link-to-broker");
                                trace!("\t\t|  |  {}:{}", name, this_link.lookup_id()?);
                                s.send(enc.clone())?;
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

