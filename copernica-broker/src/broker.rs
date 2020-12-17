use {
    crate::{
        bloom_filter::Blooms,
        router::Router,
        Bayes,
    },
    copernica_common::{LinkId, InterLinkPacket},
    anyhow::{anyhow, Result},
    crossbeam_channel::{unbounded, Receiver, Sender},
    std::collections::HashMap,
    log::{
        error, trace,
        debug
    },
};

/*
     s = Service, l = Link, b = Broker, r = Router, 2 = to: e.g. l2b = "link to copernica_broker"
     link::{udp, mpsc_channel, mpsc_corruptor, etc}
                                                            +----------------------------+
    +-----------+s2l_tx   s2l_rx+-----------+l2b_tx   l2b_rx| b2r_tx           b2r_rx    |   +-----------+   +-----------+
    |           +-------------->+           +-------------->-------------------------+   +-->+           +-->+           |
    | Service   |l2s_rx   l2s_tx|   Link    |b2l_rx   b2l_tx| r2b_rx       r2b_tx    |   |   |   Link    |   | Service   |
    |           +<--------------+           +<---------------<-------------------+   |   +<--+           +<--+           |
    +-----------+               +-----------+               |                    |   v   |   +-----------+   +-----------+
                                                            |                +---+---+-+ |
    +-----------+s2l_tx   s2l_rx+-----------+l2b_tx   l2b_rx| b2r_tx   b2r_rx|         | |   +-----------+   +-----------+
    |           +-------------->+           +-------------->---------------->+         | +-->+           +-->+           |
    | Service   |l2s_rx   l2s_tx|   Link    |b2l_rx   b2l_tx| r2b_rx   r2b_tx|  Router | |   |   Link    |   |  Broker   |
    |           +<--------------+           +<---------------<---------------+         | +<--+           +<--+           |
    +-----------+               +-----------+               |                |         | |   +-----------+   +-----------+
                                                            |                +---+---+-+ |
    +-----------+b2l_tx   b2l_rx+-----------+l2b_tx   l2b_rx| b2r_tx      b2r_rx ^   |   |   +-----------+   +-----------+
    |           +-------------->+           +-------------->---------------------+   |   +-->+           +-->+           |
    |  Broker   |l2b_rx   l2b_tx|   Link    |b2l_rx   b2l_tx| r2b_rx          r2b_tx |   |   |   Link    |   | Service   |
    |           +<--------------+           +<---------------<-----------------------+   +<--+           +<--+           |
    +-----------+               +-----------+               |           Broker           |   +-----------+   +-----------+
                                                            +----------------------------+
*/

#[derive(Clone)]
pub struct Broker {
    rs: sled::Db,
    l2b_tx: Sender<InterLinkPacket>,   // give to link
    l2b_rx: Receiver<InterLinkPacket>, // keep in broker
    b2l: HashMap<
        u32,
        (
            Sender<InterLinkPacket>,   // keep in broker
            Receiver<InterLinkPacket>, // give to link
        ),
    >,
    r2b_tx: Sender<InterLinkPacket>,   // give to router
    r2b_rx: Receiver<InterLinkPacket>, // keep in broker
    blooms: HashMap<LinkId, Blooms>,
}

impl Broker {
    pub fn new(rs: sled::Db) -> Self {
        let (l2b_tx, l2b_rx) = unbounded::<InterLinkPacket>();
        let (r2b_tx, r2b_rx) = unbounded::<InterLinkPacket>();
        let b2l = HashMap::new();
        let blooms = HashMap::new();
        Self {
            rs,
            l2b_tx,
            l2b_rx,
            r2b_tx,
            r2b_rx,
            b2l,
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
                let (b2l_tx, b2l_rx) = unbounded::<InterLinkPacket>();
                self.b2l.insert(link_id.lookup_id()?, (b2l_tx.clone(), b2l_rx.clone()));
                trace!("ADDING REMOTE: {:?}", link_id);
                self.blooms.insert(link_id, Blooms::new());
                Ok((self.l2b_tx.clone(), b2l_rx))
            }
        }
    }

    #[allow(unreachable_code)]
    pub fn run(&mut self) -> Result<()> {
        let l2b_rx = self.l2b_rx.clone();
        let mut blooms = self.blooms.clone();
        let choke = LinkId::choke();
        let b2l = self.b2l.clone();
        let r2b_tx = self.r2b_tx.clone();
        let r2b_rx = self.r2b_rx.clone();
        let mut bayes = Bayes::new();
        for (link_id, _) in &blooms {
            bayes.add_link(&link_id);
        }
        let rs = self.rs.clone();
        std::thread::spawn(move || {
            loop {
                match l2b_rx.recv() {
                    Ok(ilp) => {
                        debug!("\t\t|  |  |  broker-to-router");
                        //debug!("\t|  |  link-to-broker");
                        if !blooms.contains_key(&ilp.link_id()) {
                            trace!("ADDING {:?} to BLOOMS", ilp);
                            blooms.insert(ilp.link_id(), Blooms::new());
                            bayes.add_link(&ilp.link_id());
                        }
                        Router::handle_packet(&ilp, r2b_tx.clone(), rs.clone(), &mut blooms, &mut bayes, &choke)?;
                        while !r2b_rx.is_empty() {
                            let ilp = r2b_rx.recv()?;
                            if let Some((b2l_tx, _)) = b2l.get(&ilp.link_id().lookup_id()?) {
                                debug!("\t\t|  |  |  router-to-broker");
                                //debug!("\t|  |  broker-to-link");
                                b2l_tx.send(ilp)?;
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
