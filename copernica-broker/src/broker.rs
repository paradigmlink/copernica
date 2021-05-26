use {
    crate::{
        bloom_filter::Blooms,
        router::Router,
        Bayes,
    },
    copernica_common::{LinkId, InterLinkPacket, constants},
    anyhow::{anyhow, Result},
    futures::{
        stream::{StreamExt},
        channel::mpsc::{UnboundedSender, UnboundedReceiver, Sender, Receiver, channel, unbounded},
        sink::{SinkExt},
    },
    futures_lite::{future},
    std::{
      collections::HashMap,
    },
    async_executor::{Executor},
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

//#[derive(Clone)]
pub struct Broker {
    rs: sled::Db,
    l2b_tx: Sender<InterLinkPacket>,                           // give to link
    l2b_rx: Receiver<InterLinkPacket>,             // keep in broker
    b2l:    HashMap<u32, Sender<InterLinkPacket>>,             // keep in broker
    r2b_tx: UnboundedSender<InterLinkPacket>,                  // give to router
    r2b_rx: UnboundedReceiver<InterLinkPacket>,    // keep in broker
    blooms: HashMap<LinkId, Blooms>,
}

impl Broker {
    pub fn new(rs: sled::Db) -> Self {
        let (l2b_tx, l2b_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
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

    pub fn peer_with_link(
        &mut self,
        link_id: LinkId,
    ) -> Result<(Sender<InterLinkPacket>, Receiver<InterLinkPacket>)> {
        match self.blooms.get(&link_id) {
            Some(_) => Err(anyhow!("Channel already initialized")),
            None => {
                let (b2l_tx, b2l_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
                self.b2l.insert(link_id.lookup_id()?, b2l_tx.clone());
                trace!("ADDING REMOTE: {:?}", link_id);
                self.blooms.insert(link_id, Blooms::new());
                Ok((self.l2b_tx.clone(), b2l_rx))
            }
        }
    }

    #[allow(unreachable_code)]
    pub fn run(self) -> Result<()> {
        let mut l2b_rx = self.l2b_rx;
        let mut blooms = self.blooms.clone();
        let choke = LinkId::choke();
        let mut b2l = self.b2l.clone();
        let r2b_tx = self.r2b_tx.clone();
        let mut r2b_rx = self.r2b_rx;
        let mut bayes = Bayes::new();
        for (link_id, _) in &blooms {
            bayes.add_link(&link_id);
        }
        let rs = self.rs.clone();
//        std::thread::spawn(move || {
        let ex = Executor::new();
        let task = ex.spawn(async move {
            loop {
                match l2b_rx.next().await {
                    Some(ilp) => {
                        debug!("\t\t|  |  |  broker-to-router");
                        if !blooms.contains_key(&ilp.link_id()) {
                            trace!("ADDING {:?} to BLOOMS", ilp);
                            blooms.insert(ilp.link_id(), Blooms::new());
                            bayes.add_link(&ilp.link_id());
                        }
                        Router::handle_packet(&ilp, r2b_tx.clone(), rs.clone(), &mut blooms, &mut bayes, &choke)?;
                        if let Some(ilp) = r2b_rx.next().await {
                            if let Some(b2l_tx) = b2l.get_mut(&ilp.link_id().lookup_id()?) {
                                debug!("\t\t|  |  |  router-to-broker");
                                match b2l_tx.send(ilp).await {
                                    Ok(_) => {},
                                    Err(e) => error!("broker {:?}", e),
                                }
                            }
                        }
                        /*
                        while !r2b_rx_ref.is_empty() {
                            let ilp = r2b_rx_ref.try_next()?;
                            if let Some(b2l_tx) = b2l.get(&ilp.link_id().lookup_id()?) {
                                debug!("\t\t|  |  |  router-to-broker");
                                b2l_tx.send(ilp)?;
                            }
                        }*/
                    }
                    None => {},
                    //Err(error) => error!("{}", anyhow!("{}", error)),
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        std::thread::spawn(move || future::block_on(ex.run(task)));
        Ok(())
    }
}
