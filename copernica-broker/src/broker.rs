use {
    crate::{
        bloom_filter::Blooms,
        router::Router,
        Bayes,
    },
    copernica_common::{LinkId, InterLinkPacket, NarrowWaistPacket, constants, Operations },
    anyhow::{anyhow, Result},
    crossbeam_channel::{bounded, Receiver, Sender},
    uluru::LRUCache,
    std::{
        collections::HashMap,
    },
    log::{
        error, trace,
        //debug
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
pub type ResponseStore = LRUCache<NarrowWaistPacket, { constants::RESPONSE_STORE_SIZE }>;
pub struct Broker {
    label:  String,
    ops: Operations,
    rs:     ResponseStore,
    l2b_tx: Sender<InterLinkPacket>,                         // give to link
    l2b_rx: Receiver<InterLinkPacket>,                       // keep in broker
    b2l:    HashMap<u32, Sender<InterLinkPacket>>,           // keep in broker
    r2b_tx: Sender<InterLinkPacket>,                // give to router
    r2b_rx: Receiver<InterLinkPacket>,  // keep in broker
    blooms: HashMap<LinkId, Blooms>,
}
impl Broker {
    pub fn new((label, ops): (String, Operations)) -> Self {
        let (l2b_tx, l2b_rx) = bounded::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (r2b_tx, r2b_rx) = bounded::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let b2l = HashMap::new();
        let blooms = HashMap::new();
        let rs = ResponseStore::default();
        ops.register_router(label.clone());
        Self {
            label,
            rs,
            l2b_tx,
            l2b_rx,
            r2b_tx,
            r2b_rx,
            b2l,
            blooms,
            ops,
        }
    }
    pub fn peer_with_link(
        &mut self,
        link_id: LinkId,
    ) -> Result<(Sender<InterLinkPacket>, Receiver<InterLinkPacket>)> {
        match self.blooms.get(&link_id) {
            Some(_) => Err(anyhow!("Channel already initialized")),
            None => {
                let (b2l_tx, b2l_rx) = bounded::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
                self.b2l.insert(link_id.lookup_id()?, b2l_tx.clone());
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
        let mut b2l = self.b2l.clone();
        let r2b_tx = self.r2b_tx.clone();
        let r2b_rx = self.r2b_rx.clone();
        let mut bayes = Bayes::new();
        for (link_id, _) in &blooms {
            bayes.add_link(&link_id);
        }
        let rs = self.rs.clone();
        let ops = self.ops.clone();
        let label = self.label.clone();
        std::thread::spawn(move || {
            loop {
                match l2b_rx.recv() {
                    Ok(ilp) => {
                        trace!("\t\t|  |  |  broker-to-router");
                        ops.message_from(label.clone());
                        if !blooms.contains_key(&ilp.link_id()) {
                            trace!("ADDING {:?} to BLOOMS", ilp);
                            blooms.insert(ilp.link_id(), Blooms::new());
                            bayes.add_link(&ilp.link_id());
                        }
                        Router::handle_packet(&label, &ops, &ilp, r2b_tx.clone(), &mut rs.clone(), &mut blooms, &mut bayes, &choke)?;
                    }
                    Err(error) => error!("{}", error),
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        let ops = self.ops.clone();
        let label = self.label.clone();
        std::thread::spawn(move || {
            loop {
                if let Ok(ilp) = r2b_rx.recv() {
                    match &ilp.link_id().lookup_id() {
                        Ok(id) => {
                            match b2l.get_mut(id) {
                                Some(b2l_tx) => {
                                    trace!("\t\t|  |  |  router-to-broker");
                                    ops.message_from(label.clone());
                                    match b2l_tx.send(ilp) {
                                        Ok(_) => {},
                                        Err(e) => error!("broker {:?}", e),
                                    }
                                },
                                None => { continue }
                            }
                        },
                        Err(_e) => { continue },
                    };
                }
            }
        });
        Ok(())
    }
}
