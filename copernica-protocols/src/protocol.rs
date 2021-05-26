use {
    copernica_common::{LinkId, NarrowWaistPacket, LinkPacket, InterLinkPacket, HBFI, serialization::*, PrivateIdentityInterface,
    constants},
    log::{debug, error},
    futures::{
        stream::{StreamExt},
        channel::mpsc::{Sender, Receiver, channel},
        sink::{SinkExt},
        lock::Mutex,
    },
    sled::{Db, Event},
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
pub struct Inbound {
    pub l2p_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
}
impl Inbound {
    pub fn new(l2p_rx: Receiver<InterLinkPacket>) -> Self {
        Self { l2p_rx: Arc::new(Mutex::new(l2p_rx)) }
    }
    pub async fn next_inbound(self) -> Option<InterLinkPacket> {
        let l2p_rx_mutex = Arc::clone(&self.l2p_rx);
        let mut l2p_rx_ref = l2p_rx_mutex.lock().await;
        l2p_rx_ref.next().await
    }
}
#[derive(Clone)]
pub struct Outbound {
    pub db: Db,
    pub link_id: LinkId,
    pub protocol_sid: PrivateIdentityInterface,
    pub p2l_tx: Sender<InterLinkPacket>,
}
impl Outbound {
    pub fn new(db: Db, link_id: LinkId, protocol_sid: PrivateIdentityInterface, p2l_tx: Sender<InterLinkPacket>) -> Self {
        Self {db, link_id, protocol_sid, p2l_tx}
    }
    /*
    pub fn request(&self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<u8>> {
        let mut counter = start;
        let mut reconstruct: Vec<Result<Vec<u8>>> = vec![];
        let mut futures_reconstruct = vec![];
        while counter <= end {
            let hbfi = hbfi.clone().offset(counter);
            let (_, hbfi_s) = serialize_hbfi(&hbfi)?;
            if let None = self.db.get(hbfi_s.clone())? {
                let nw = NarrowWaistPacket::request(hbfi.clone())?;
                let lp = LinkPacket::new(self.link_id.reply_to()?, nw);
                let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
                let subscriber = self.db.watch_prefix(hbfi_s);
                debug!("\t\t|  protocol-to-link");
                futures_reconstruct.push(process_subscriber(subscriber, self.protocol_sid.clone()));
                self.p2l_tx.try_send(ilp)?;
            }
            counter += 1;
        }
        task::block_on(async {
            reconstruct = join_all(futures_reconstruct).await;
        });
        debug!("HERE WE ARE {:?}", reconstruct);
        let reconstructed = reconstruct.into_iter().map(|u|u.unwrap()).flatten().collect();
        Ok(reconstructed)
    }*/
    pub async fn request2(&mut self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<u8>> {
        let mut counter = start;
        let mut reconstruct: Vec<u8> = vec![];
        while counter <= end {
            let hbfi = hbfi.clone().offset(counter);
            let (_, hbfi_s) = serialize_hbfi(&hbfi)?;
            match self.db.get(hbfi_s.clone())? {
                Some(resp) => {
                    let nw = deserialize_narrow_waist_packet(&resp.to_vec())?;
                    let chunk = match hbfi.request_pid {
                        Some(_) => {
                            nw.data(Some(self.protocol_sid.clone()))?
                        },
                        None => {
                            nw.data(None)?
                        },
                    };
                    reconstruct.append(&mut chunk.clone());
                }
                None => {
                    let nw = NarrowWaistPacket::request(hbfi.clone())?;
                    let lp = LinkPacket::new(self.link_id.reply_to()?, nw);
                    let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
                    let subscriber = self.db.watch_prefix(hbfi_s);
                    debug!("\t\t|  protocol-to-link");
                    match self.p2l_tx.send(ilp).await {
                        Ok(_) => {},
                        Err(e) => error!("protocol send error {:?}", e),
                    }
                    for event in subscriber.take(1) {
                        match event {
                            Event::Insert{ key: _, value } => {
                                let nw = deserialize_narrow_waist_packet(&value.to_vec())?;
                                match nw.clone() {
                                    NarrowWaistPacket::Request {..} => continue,
                                    NarrowWaistPacket::Response { hbfi: hbfi_inbound, .. } => {
                                        if hbfi.to_bfis() == hbfi_inbound.to_bfis() {
                                            let chunk = match hbfi_inbound.request_pid {
                                                Some(_) => {
                                                    nw.data(Some(self.protocol_sid.clone()))?
                                                },
                                                None => {
                                                    nw.data(None)?
                                                },
                                            };
                                            //debug!("value {:?}", chunk);
                                            reconstruct.append(&mut chunk.clone());
                                        } else {
                                            continue
                                        }
                                    }
                                }
                            }
                            Event::Remove {key:_ } => {}
                        }
                    }
                }
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
        //self.p2l_tx.clone().send(ilp.clone()).await?;
        match self.p2l_tx.send(ilp).await {
            Ok(_) => {},
            Err(e) => error!("protocol send error {:?}", e),
        }
        Ok(())
    }
}
/*
async fn process_subscriber(mut subscriber: Subscriber, sid: PrivateIdentityInterface) -> Result<Vec<u8>> {
    let chunk = vec![];
    while let Some(event) = (&mut subscriber).await {
    //for event in subscriber.take(1) {
        match event {
            Event::Insert{ key, value } => {
                //let _key_s: HBFI = deserialize_cyphertext_hbfi(&key.to_vec())?;
                debug!("HBFI {:?}", key);
                let nw = deserialize_narrow_waist_packet(&value.to_vec())?;
                    match nw.clone() {
                        NarrowWaistPacket::Request {..} => {},
                        NarrowWaistPacket::Response {hbfi, ..} => {
                            match hbfi.request_pid {
                                Some(_) => {
                                    nw.data(Some(sid.clone()))?
                                },
                                None => {
                                    nw.data(None)?
                                },
                            };
                        }
                    }
            }
            Event::Remove {key:_ } => {}
        }
    }
    Ok(chunk)
}
*/
pub trait Protocol<'a> {
    fn get_db(&mut self) -> sled::Db;
    fn get_protocol_sid(&mut self) -> PrivateIdentityInterface;
    fn set_outbound(&mut self, outbound: Outbound);
    fn set_inbound(&mut self, inbound: Inbound);
    fn peer_with_link(&mut self, link_id: LinkId) -> Result<(Sender<InterLinkPacket>, Receiver<InterLinkPacket>)> {
        let (l2p_tx, l2p_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (p2l_tx, p2l_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let inbound = Inbound::new(l2p_rx);
        let outbound = Outbound::new(self.get_db(), link_id, self.get_protocol_sid(), p2l_tx);
        self.set_inbound(inbound);
        self.set_outbound(outbound);
        Ok((l2p_tx, p2l_rx))
    }
    #[allow(unreachable_code)]
    fn run(&self) -> Result<()>;
    fn new(db: sled::Db, protocol_sid: PrivateIdentityInterface) -> Self where Self: Sized;
}


