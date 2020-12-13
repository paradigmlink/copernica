use {
    crate::{Link, decode, encode},
    copernica_common::{
        InterLinkPacket, LinkId, ReplyTo
    },
    anyhow::{anyhow, Result},
    crossbeam_channel::{Sender, Receiver, unbounded},
    log::{debug, error, trace},
};

pub struct MpscCorruptor {
    name: String,
    link_id: LinkId,
    // t = tansport; c = copernic; 0 = this instance of t; 1 = the pair of same type
    t2c_tx: Sender<InterLinkPacket>,
    c2t_rx: Receiver<InterLinkPacket>,
    t2t0_tx: Sender<Vec<u8>>,        // give
    t2t0_rx: Receiver<Vec<u8>>,      // keep
    t2t1_tx: Option<Vec<Sender<Vec<u8>>>>,
}

impl MpscCorruptor {
    pub fn male(&self) -> Sender<Vec<u8>> {
        self.t2t0_tx.clone()
    }
    pub fn female(&mut self, new_t2t1_tx: Sender<Vec<u8>>) {
        if let None = self.t2t1_tx {
            self.t2t1_tx = Some(vec![]);
        }
        if let Some(t2t1_tx) = &mut self.t2t1_tx {
            t2t1_tx.push(new_t2t1_tx);
        }
    }
}

impl<'a> Link<'a> for MpscCorruptor {
    fn new(name: String
        , link_id: LinkId
        , (t2c_tx, c2t_rx): ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> )
        ) -> Result<MpscCorruptor> {
        match link_id.reply_to()? {
            ReplyTo::Mpsc => {
                let (t2t0_tx, t2t0_rx) = unbounded::<Vec<u8>>();
                return Ok(
                    MpscCorruptor {
                        name,
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
    fn run(&self) -> Result<()> {
        let name = self.name.clone();
        let this_link = self.link_id.clone();
        trace!("Started {:?}:", this_link);
        let t2t0_rx = self.t2t0_rx.clone();
        let t2c_tx = self.t2c_tx.clone();
        std::thread::spawn(move || {
            match this_link.reply_to()? {
                ReplyTo::Mpsc => {
                    loop {
                        match t2t0_rx.recv(){
                            Ok(msg) => {
                                let (lnk_tx_pid, lp) = match this_link.rx_pid()? {
                                    Some(_) => decode(msg, Some(this_link.sid()?))?,
                                    None => decode(msg, None)?,
                                };
                                let link_id = LinkId::new(this_link.lookup_id()?, this_link.sid()?, this_link.rx_pid()?, lp.reply_to());
                                let ilp = InterLinkPacket::new(link_id, lp);
                                debug!("{}:{}", name, this_link.lookup_id()?);
                                let _r = t2c_tx.send(ilp)?;
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
        let c2t_rx = self.c2t_rx.clone();
        if let Some(t2t1_tx) = self.t2t1_tx.clone() {
            std::thread::spawn(move || {
                loop {
                    match c2t_rx.recv(){
                        Ok(ilp) => {
                            let lp = ilp.link_packet().change_origination(this_link.reply_to()?);
                            let enc = match this_link.rx_pid()? {
                                Some(lnk_rx_pid) => encode(lp, this_link.sid()?, Some(lnk_rx_pid))?,
                                None => encode(lp, this_link.sid()?, None)?,
                            };
                            let mut corrupted = enc;
                            for i in 4..10 {
                                corrupted[i] = 0x0;
                            }
                            for s in t2t1_tx.clone() {
                                debug!("{}:{}", name, this_link.lookup_id()?);
                                s.send(corrupted.clone())?;
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

