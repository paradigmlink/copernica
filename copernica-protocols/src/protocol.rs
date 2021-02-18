use {
    copernica_common::{LinkId, NarrowWaistPacket, LinkPacket, InterLinkPacket, HBFI, serialization::*},
    copernica_identity::{PrivateIdentity},
    //std::{thread},
    log::{debug, error},
    crossbeam_channel::{Sender, Receiver, unbounded},
    sled::{Db, Event, Subscriber},
    anyhow::{Result, anyhow},
    futures::future::{join_all},
    async_std::{task},
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
    pub db: Db,
    pub link_id: LinkId,
    pub sid: PrivateIdentity,
    pub p2l_tx: Sender<InterLinkPacket>,
    pub l2p_rx: Receiver<InterLinkPacket>,
}
async fn process_subscriber(mut subscriber: Subscriber, sid: PrivateIdentity) -> Result<Vec<u8>> {
    let mut chunk = vec![];
    while let Some(event) = (&mut subscriber).await {
    //for event in subscriber.take(1) {
        match event {
            Event::Insert{ key: key, value } => {
                let key_s: HBFI = deserialize_cyphertext_hbfi(&key.to_vec())?;
                debug!("HBFI {:?}", key);
                let nw = deserialize_narrow_waist_packet(&value.to_vec())?;
                    match nw.clone() {
                        NarrowWaistPacket::Request {..} => {},
                        NarrowWaistPacket::Response {hbfi, data, ..} => {
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
impl TxRx {
    pub fn new(db: Db, link_id: LinkId, sid: PrivateIdentity, p2l_tx: Sender<InterLinkPacket>, l2p_rx: Receiver<InterLinkPacket>) -> Self {
        Self {db, link_id, sid, p2l_tx, l2p_rx}
    }
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
                futures_reconstruct.push(process_subscriber(subscriber, self.sid.clone()));
                self.p2l_tx.send(ilp)?;
            }
            counter += 1;
        }
        task::block_on(async {
            reconstruct = join_all(futures_reconstruct).await;
        });
        debug!("HERE WE ARE {:?}", reconstruct);
        let reconstructed = reconstruct.into_iter().map(|u|u.unwrap()).flatten().collect();
        Ok(reconstructed)
    }
    pub fn request2(&self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<u8>> {
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
                            nw.data(Some(self.sid.clone()))?
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
                    self.p2l_tx.send(ilp)?;
                    //debug!("                                HELLO");
                    for event in subscriber.take(1) {
                        //debug!("                            HELLO");
                        //debug!("{:?}", event);
                        match event {
                            Event::Insert{ key: _, value } => {
                                let nw = deserialize_narrow_waist_packet(&value.to_vec())?;
                                match nw.clone() {
                                    NarrowWaistPacket::Request {..} => continue,
                                    NarrowWaistPacket::Response { hbfi: hbfi_inbound, .. } => {
                                        if hbfi.to_bfis() == hbfi_inbound.to_bfis() {
                                            let chunk = match hbfi_inbound.request_pid {
                                                Some(_) => {
                                                    nw.data(Some(self.sid.clone()))?
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
    pub fn respond(&self,
        hbfi: HBFI,
        data: Vec<u8>,
    ) -> Result<()> {
        debug!("\t\t|  RESPONSE PACKET FOUND");
        let nw = NarrowWaistPacket::response(self.sid.clone(), hbfi.clone(), data, 0, 0)?;
        let lp = LinkPacket::new(self.link_id.reply_to()?, nw);
        let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
        debug!("\t\t|  protocol-to-link");
        self.p2l_tx.send(ilp.clone())?;
        Ok(())
    }
}

pub trait Protocol<'a> {
    fn set_txrx(&mut self, txrx: TxRx);
    fn get_txrx(&mut self) -> Option<TxRx>;
    fn peer_with_link(
        &mut self,
        db: sled::Db,
        link_id: LinkId,
        sid: PrivateIdentity
    ) -> Result<(Sender<InterLinkPacket>, Receiver<InterLinkPacket>)> {
        let (l2p_tx, l2p_rx) = unbounded::<InterLinkPacket>();
        let (p2l_tx, p2l_rx) = unbounded::<InterLinkPacket>();
        let txrx = TxRx::new(db, link_id, sid, p2l_tx, l2p_rx);
        self.set_txrx(txrx);
        Ok((l2p_tx, p2l_rx))
    }
    #[allow(unreachable_code)]
    fn run(&mut self) -> Result<()>;
    fn new() -> Self where Self: Sized; //kept at end cause amp syntax highlighting falls over on the last :
}


