use {
    crate::{
        bloom_filter::Blooms,
        router::Router,
        Bayes,
    },
    copernica_common::{LinkId, InterLinkPacket, NarrowWaistPacket, constants},
    anyhow::{anyhow, Result},
    futures::{ join,
        stream::{StreamExt},
        channel::mpsc::{UnboundedSender, UnboundedReceiver, Sender, Receiver, channel, unbounded},
        sink::{SinkExt},
        lock::Mutex,
    },
    futures_lite::{future},
    uluru::LRUCache,
    std::{
        collections::HashMap,
        sync::{Arc},
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
pub type ResponseStore = LRUCache<NarrowWaistPacket, { constants::RESPONSE_STORE_SIZE }>;
pub struct Broker {
    rs:     ResponseStore,
    l2b_tx: Sender<InterLinkPacket>,                         // give to link
    l2b_rx: Receiver<InterLinkPacket>,                       // keep in broker
    b2l:    HashMap<u32, Sender<InterLinkPacket>>,           // keep in broker
    r2b_tx: UnboundedSender<InterLinkPacket>,                // give to router
    r2b_rx: Arc<Mutex<UnboundedReceiver<InterLinkPacket>>>,  // keep in broker
    blooms: HashMap<LinkId, Blooms>,
}
impl Broker {
    pub fn new() -> Self {
        let (l2b_tx, l2b_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (r2b_tx, r2b_rx) = unbounded::<InterLinkPacket>();
        let b2l = HashMap::new();
        let blooms = HashMap::new();
        let rs = ResponseStore::default();
        Self {
            rs,
            l2b_tx,
            l2b_rx,
            r2b_tx,
            r2b_rx: Arc::new(Mutex::new(r2b_rx)),
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
        let r2b_rx_mutex = Arc::clone(&self.r2b_rx);
        let mut bayes = Bayes::new();
        for (link_id, _) in &blooms {
            bayes.add_link(&link_id);
        }
        let rs = self.rs.clone();
        let ex = Executor::new();
        let receiver_task = ex.spawn(async move {
            loop {
                match l2b_rx.next().await {
                    Some(ilp) => {
                        debug!("\t\t|  |  |  broker-to-router");
                        if !blooms.contains_key(&ilp.link_id()) {
                            trace!("ADDING {:?} to BLOOMS", ilp);
                            blooms.insert(ilp.link_id(), Blooms::new());
                            bayes.add_link(&ilp.link_id());
                        }
                        Router::handle_packet(&ilp, r2b_tx.clone(), &mut rs.clone(), &mut blooms, &mut bayes, &choke)?;
                    }
                    None => {},
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        let sender_task = async move {
            loop {
                let r2b_rx_mutex = r2b_rx_mutex.clone();
                let mut r2b_rx_ref = r2b_rx_mutex.lock().await;
                if let Some(ilp) = r2b_rx_ref.next().await {
                    match &ilp.link_id().lookup_id() {
                        Ok(id) => {
                            match b2l.get_mut(id) {
                                Some(b2l_tx) => {
                                    debug!("\t\t|  |  |  router-to-broker");
                                    future::block_on(async {
                                        match b2l_tx.send(ilp).await {
                                            Ok(_) => {},
                                            Err(e) => error!("broker {:?}", e),
                                        }
                                    });
                                },
                                None => { continue }
                            }
                        },
                        Err(_e) => { continue },
                    };
                }
            }
        };
        std::thread::spawn(move || future::block_on(ex.run(async { join!(receiver_task, sender_task) })));
        Ok(())
    }
}
