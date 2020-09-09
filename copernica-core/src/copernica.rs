use {
    crate::{
        link::{Blooms, Nonce, LinkId},
        packets::InterLinkPacket,
        router::Router,
    },
    anyhow::{anyhow, Result},
    crossbeam_channel::{unbounded, Receiver, Sender},
    std::collections::HashMap,
    log::{
        error, trace,
        //debug
    },
};

#[derive(Clone)]
pub struct Copernica {
    // t = transport, c = copernica, r = router
    t2c_tx: Sender<InterLinkPacket>,   // give to transports
    t2c_rx: Receiver<InterLinkPacket>, // keep in copernica
    c2t: HashMap<
        Nonce,
        (
            Sender<InterLinkPacket>,   // keep in copernica
            Receiver<InterLinkPacket>, // give to transports
        ),
    >,
    r2c_tx: Sender<InterLinkPacket>,   // give to router
    r2c_rx: Receiver<InterLinkPacket>, // keep in copernica
    blooms: HashMap<LinkId, Blooms>,
}

impl Copernica {
    pub fn new() -> Self {
        let (t2c_tx, t2c_rx) = unbounded::<InterLinkPacket>();
        let (r2c_tx, r2c_rx) = unbounded::<InterLinkPacket>();
        let c2t = HashMap::new();
        let blooms = HashMap::new();
        Self {
            t2c_tx,
            t2c_rx,
            r2c_tx,
            r2c_rx,
            c2t,
            blooms,
        }
    }

    pub fn peer(
        &mut self,
        link_id: LinkId,
    ) -> Result<(Sender<InterLinkPacket>, Receiver<InterLinkPacket>)> {
        match self.blooms.get(&link_id) {
            Some(_) => Err(anyhow!("Channel already initialized")),
            None => {
                let (c2t_tx, c2t_rx) = unbounded::<InterLinkPacket>();
                self.c2t.insert(link_id.nonce(), (c2t_tx.clone(), c2t_rx.clone()));
                trace!("ADDING REMOTE: {:?}", link_id);
                self.blooms.insert(link_id, Blooms::new());
                Ok((self.t2c_tx.clone(), c2t_rx))
            }
        }
    }

    #[allow(unreachable_code)]
    pub fn run(&mut self, db: sled::Db) -> Result<()> {
        //trace!("{:?} IS LISTENING", self.listen_addr);
        let t2c_rx = self.t2c_rx.clone();
        let mut blooms = self.blooms.clone();
        let c2t = self.c2t.clone();
        let r2c_tx = self.r2c_tx.clone();
        let r2c_rx = self.r2c_rx.clone();
        std::thread::spawn(move || {
            loop {
                match t2c_rx.recv() {
                    Ok(ilp) => {
                        if !blooms.contains_key(&ilp.link_id()) {
                            trace!("ADDING {:?} to BLOOMS", ilp);
                            blooms.insert(ilp.link_id(), Blooms::new());
                        }
                        Router::handle_packet(&ilp, r2c_tx.clone(), db.clone(), &mut blooms)?;
                        while !r2c_rx.is_empty() {
                            let ilp = r2c_rx.recv()?;
                            if let Some((c2t_tx, _)) = c2t.get(&ilp.link_id().nonce()) {
                                c2t_tx.send(ilp)?;
                            }
                        }
                    }
                    Err(error) => error!("{}", anyhow!("{}", error)),
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }
}
