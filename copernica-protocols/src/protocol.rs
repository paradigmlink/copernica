use {
    copernica_common::{LinkId, NarrowWaistPacket, LinkPacket, InterLinkPacket, HBFI, PrivateIdentityInterface,
    constants},
    log::{debug, error},
    futures::{
        stream::{StreamExt},
        channel::mpsc::{Sender, Receiver, channel},
        sink::{SinkExt},
        lock::Mutex,
    },
    anyhow::{Result},
    std::sync::{Arc},
};

/*
     s = Protocol, l = Link, b = Broker, r = Router, 2 = to: e.g. l2b = "link to copernica_broker"
     link::{udp, mpsc_channel, mpsc_corruptor, etc}
                                                            +----------------------------+
    +-----------+p2l_tx   p2l_rx+-----------+l2b_tx   l2b_rx| b2r_tx           b2r_rx    |   +-----------+   +-----------+
    |           +-------------->+           +-------------->-------------------------+   +-->+           +-->+           |
    | Protocol   |l2p_rx   l2p_tx|   Link    |b2l_rx   b2l_tx| r2b_rx       r2b_tx    |   |   |   Link    |   | Protocol   |
    |           +<--------------+           +<---------------<-------------------+   |   +<--+           +<--+           |
    +-----------+               +-----------+               |                    |   v   |   +-----------+   +-----------+
                                                            |                +---+---+-+ |
    +-----------+p2l_tx   p2l_rx+-----------+l2b_tx   l2b_rx| b2r_tx   b2r_rx|         | |   +-----------+   +-----------+
    |           +-------------->+           +-------------->---------------->+         | +-->+           +-->+           |
    | Protocol   |l2p_rx   l2p_tx|   Link    |b2l_rx   b2l_tx| r2b_rx   r2b_tx|  Router | |   |   Link    |   |  Broker   |
    |           +<--------------+           +<---------------<---------------+         | +<--+           +<--+           |
    +-----------+               +-----------+               |                |         | |   +-----------+   +-----------+
                                                            |                +---+---+-+ |
    +-----------+b2l_tx   b2l_rx+-----------+l2b_tx   l2b_rx| b2r_tx      b2r_rx ^   |   |   +-----------+   +-----------+
    |           +-------------->+           +-------------->---------------------+   |   +-->+           +-->+           |
    |  Broker   |l2b_rx   l2b_tx|   Link    |b2l_rx   b2l_tx| r2b_rx          r2b_tx |   |   |   Link    |   | Protocol   |
    |           +<--------------+           +<---------------<-----------------------+   +<--+           +<--+           |
    +-----------+               +-----------+               |           Broker           |   +-----------+   +-----------+
                                                            +----------------------------+
*/
#[derive(Clone)]
pub struct TxRx {
    pub link_id: LinkId,
    pub protocol_sid: PrivateIdentityInterface,
    pub p2l_tx: Sender<InterLinkPacket>,
    pub l2p_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
    pub unreliable_unordered_response_tx: Sender<InterLinkPacket>,
    pub unreliable_unordered_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
    pub unreliable_sequenced_response_tx: Sender<InterLinkPacket>,
    pub unreliable_sequenced_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
    pub reliable_unordered_response_tx: Sender<InterLinkPacket>,
    pub reliable_unordered_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
    pub reliable_ordered_response_tx: Sender<InterLinkPacket>,
    pub reliable_ordered_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
    pub reliable_sequenced_response_tx: Sender<InterLinkPacket>,
    pub reliable_sequenced_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
}
impl TxRx {
    pub fn new(link_id: LinkId, protocol_sid: PrivateIdentityInterface, p2l_tx: Sender<InterLinkPacket>, l2p_rx: Receiver<InterLinkPacket>) -> TxRx
    {
        let (unreliable_unordered_response_tx, unreliable_unordered_response_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (unreliable_sequenced_response_tx, unreliable_sequenced_response_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (reliable_unordered_response_tx, reliable_unordered_response_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (reliable_ordered_response_tx, reliable_ordered_response_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (reliable_sequenced_response_tx, reliable_sequenced_response_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        TxRx {
            link_id,
            protocol_sid,
            p2l_tx,
            l2p_rx: Arc::new(Mutex::new(l2p_rx)),
            unreliable_unordered_response_rx: Arc::new(Mutex::new(unreliable_unordered_response_rx)),
            unreliable_unordered_response_tx,
            unreliable_sequenced_response_rx: Arc::new(Mutex::new(unreliable_sequenced_response_rx)),
            unreliable_sequenced_response_tx,
            reliable_unordered_response_rx: Arc::new(Mutex::new(reliable_unordered_response_rx)),
            reliable_unordered_response_tx,
            reliable_ordered_response_rx: Arc::new(Mutex::new(reliable_ordered_response_rx)),
            reliable_ordered_response_tx,
            reliable_sequenced_response_rx: Arc::new(Mutex::new(reliable_sequenced_response_rx)),
            reliable_sequenced_response_tx,
         }
    }
    pub async fn next_inbound(self) -> Option<InterLinkPacket> {
        let l2p_rx_mutex = Arc::clone(&self.l2p_rx);
        let mut l2p_rx_ref = l2p_rx_mutex.lock().await;
        l2p_rx_ref.next().await
    }
    pub async fn unreliable_unordered_request(&self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<u8>> {
        let mut counter = start;
        let mut reconstruct: Vec<u8> = vec![];
        let nw = NarrowWaistPacket::request(hbfi.clone())?;
        let lp = LinkPacket::new(self.link_id.reply_to()?, nw);
        let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
        debug!("\t\t|  protocol-to-link");
        let mut p2l_tx = self.p2l_tx.clone();
        match p2l_tx.send(ilp).await {
            Ok(_) => { },
            Err(e) => error!("protocol send error {:?}", e),
        }
        let unreliable_unordered_response_rx_mutex = Arc::clone(&self.unreliable_unordered_response_rx);
        let mut unreliable_unordered_response_rx_ref = unreliable_unordered_response_rx_mutex.lock().await;
        while counter <= end {
            match unreliable_unordered_response_rx_ref.next().await {
                Some(ilp) => {
                    let nw = ilp.narrow_waist();
                    let chunk = match hbfi.request_pid {
                        Some(_) => {
                            nw.data(Some(self.protocol_sid.clone()))?
                        },
                        None => {
                            nw.data(None)?
                        },
                    };
                    reconstruct.append(&mut chunk.clone());
                },
                None => {}
            }
            counter += 1;
        }
        Ok(reconstruct)
    }
    pub async fn unreliable_sequenced_request(&self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<u8>> {
        let mut counter = start;
        let mut reconstruct: Vec<u8> = vec![];
        let nw = NarrowWaistPacket::request(hbfi.clone())?;
        let lp = LinkPacket::new(self.link_id.reply_to()?, nw);
        let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
        debug!("\t\t|  protocol-to-link");
        let mut p2l_tx = self.p2l_tx.clone();
        match p2l_tx.send(ilp).await {
            Ok(_) => {},
            Err(e) => error!("protocol send error {:?}", e),
        }
        let unreliable_sequenced_response_rx_mutex = Arc::clone(&self.unreliable_sequenced_response_rx);
        let mut unreliable_sequenced_response_rx_ref = unreliable_sequenced_response_rx_mutex.lock().await;
        while counter <= end {
            match unreliable_sequenced_response_rx_ref.next().await {
                Some(ilp) => {
                    let nw = ilp.narrow_waist();
                    match nw.clone() {
                        NarrowWaistPacket::Request { hbfi, .. } => if hbfi.ost < counter { continue },
                        NarrowWaistPacket::Response { hbfi, .. } => if hbfi.ost < counter { continue },
                    }
                    let chunk = match hbfi.request_pid {
                        Some(_) => {
                            nw.data(Some(self.protocol_sid.clone()))?
                        },
                        None => {
                            nw.data(None)?
                        },
                    };
                    reconstruct.append(&mut chunk.clone());
                    debug!("RECONSTRUCT: {:?}, counter: {}", reconstruct, counter);
                },
                None => {
                    debug!("NOTHING");
                }
            }
            counter += 1;
            debug!("RECONSTRUCT: {:?}, counter: {}", reconstruct, counter);
        }
        Ok(reconstruct)
    }

    pub async fn reliable_unordered_request(&self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<u8>> {
        let mut counter = start;
        let mut reconstruct: Vec<u8> = vec![];
        let nw = NarrowWaistPacket::request(hbfi.clone())?;
        let lp = LinkPacket::new(self.link_id.reply_to()?, nw);
        let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
        debug!("\t\t|  protocol-to-link");
        let mut p2l_tx = self.p2l_tx.clone();
        match p2l_tx.send(ilp).await {
            Ok(_) => {},
            Err(e) => error!("protocol send error {:?}", e),
        }
        let reliable_unordered_response_rx_mutex = Arc::clone(&self.reliable_unordered_response_rx);
        let mut reliable_unordered_response_rx_ref = reliable_unordered_response_rx_mutex.lock().await;
        while counter <= end {
            match reliable_unordered_response_rx_ref.next().await {
                Some(ilp) => {
                    let nw = ilp.narrow_waist();
                    match nw.clone() {
                        NarrowWaistPacket::Request { hbfi, .. } => if hbfi.ost < counter { continue },
                        NarrowWaistPacket::Response { hbfi, .. } => if hbfi.ost < counter { continue },
                    }
                    let chunk = match hbfi.request_pid {
                        Some(_) => {
                            nw.data(Some(self.protocol_sid.clone()))?
                        },
                        None => {
                            nw.data(None)?
                        },
                    };
                    reconstruct.append(&mut chunk.clone());
                },
                None => {}
            }
            counter += 1;
        }
        Ok(reconstruct)
    }
    pub async fn reliable_ordered_request(&self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<u8>> {
        let mut counter = start;
        let mut reconstruct: Vec<u8> = vec![];
        let nw = NarrowWaistPacket::request(hbfi.clone())?;
        let lp = LinkPacket::new(self.link_id.reply_to()?, nw);
        let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
        debug!("\t\t|  protocol-to-link");
        let mut p2l_tx = self.p2l_tx.clone();
        match p2l_tx.send(ilp).await {
            Ok(_) => {},
            Err(e) => error!("protocol send error {:?}", e),
        }
        let reliable_ordered_response_rx_mutex = Arc::clone(&self.reliable_ordered_response_rx);
        let mut reliable_ordered_response_rx_ref = reliable_ordered_response_rx_mutex.lock().await;
        while counter <= end {
            match reliable_ordered_response_rx_ref.next().await {
                Some(ilp) => {
                    let nw = ilp.narrow_waist();
                    match nw.clone() {
                        NarrowWaistPacket::Request { hbfi, .. } => if hbfi.ost < counter { continue },
                        NarrowWaistPacket::Response { hbfi, .. } => if hbfi.ost < counter { continue },
                    }
                    let chunk = match hbfi.request_pid {
                        Some(_) => {
                            nw.data(Some(self.protocol_sid.clone()))?
                        },
                        None => {
                            nw.data(None)?
                        },
                    };
                    reconstruct.append(&mut chunk.clone());

                },
                None => {}
            }
            counter += 1;
        }
        Ok(reconstruct)
    }
    pub async fn reliable_sequenced_request(&self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<u8>> {
        let mut counter = start;
        let mut reconstruct: Vec<u8> = vec![];
        let nw = NarrowWaistPacket::request(hbfi.clone())?;
        let lp = LinkPacket::new(self.link_id.reply_to()?, nw);
        let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
        debug!("\t\t|  protocol-to-link");
        let mut p2l_tx = self.p2l_tx.clone();
        match p2l_tx.send(ilp).await {
            Ok(_) => {},
            Err(e) => error!("protocol send error {:?}", e),
        }
        let reliable_sequenced_response_rx_mutex = Arc::clone(&self.reliable_sequenced_response_rx);
        let mut reliable_sequenced_response_rx_ref = reliable_sequenced_response_rx_mutex.lock().await;
        while counter <= end {
            match reliable_sequenced_response_rx_ref.next().await {
                Some(ilp) => {
                    let nw = ilp.narrow_waist();
                    match nw.clone() {
                        NarrowWaistPacket::Request { hbfi, .. } => if hbfi.ost < counter { continue },
                        NarrowWaistPacket::Response { hbfi, .. } => if hbfi.ost < counter { continue },
                    }
                    let chunk = match hbfi.request_pid {
                        Some(_) => {
                            nw.data(Some(self.protocol_sid.clone()))?
                        },
                        None => {
                            nw.data(None)?
                        },
                    };
                    reconstruct.append(&mut chunk.clone());

                },
                None => {}
            }
            counter += 1;
        }
        Ok(reconstruct)
    }
    pub async fn respond(mut self,
        hbfi: HBFI,
        data: Vec<u8>,
    ) -> Result<()> {
        debug!("\t\t|  RESPONSE PACKET FOUND");
        let nw = NarrowWaistPacket::response(self.protocol_sid.clone(), hbfi.clone(), data, 0, 0)?;
        let lp = LinkPacket::new(self.link_id.reply_to()?, nw);
        let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
        debug!("\t\t|  protocol-to-link");
        match self.p2l_tx.send(ilp).await {
            Ok(_) => {},
            Err(e) => error!("protocol send error {:?}", e),
        }
        Ok(())
    }
}
pub trait Protocol<'a> {
    fn get_protocol_sid(&mut self) -> PrivateIdentityInterface;
    fn set_txrx(&mut self, txrx: TxRx);
    fn peer_with_link(&mut self, link_id: LinkId) -> Result<(Sender<InterLinkPacket>, Receiver<InterLinkPacket>)> {
        let (l2p_tx, l2p_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (p2l_tx, p2l_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let txrx = TxRx::new(link_id, self.get_protocol_sid(), p2l_tx, l2p_rx);
        self.set_txrx(txrx);
        Ok((l2p_tx, p2l_rx))
    }
    #[allow(unreachable_code)]
    fn run(&self) -> Result<()>;
    fn new(protocol_sid: PrivateIdentityInterface) -> Self where Self: Sized;
}


