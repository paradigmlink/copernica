use {
    copernica_common::{LinkId, NarrowWaistPacket, NWWhereRequestEqResponse, LinkPacket, InterLinkPacket, HBFI, HBFIWithoutFrame, PrivateIdentityInterface,
    constants, Nonce},
    log::{debug, error},
    futures::{
        stream::{StreamExt},
        channel::mpsc::{Sender, Receiver, channel},
        sink::{SinkExt},
        lock::Mutex,
    },
    waitmap::{WaitMap},
    smol_timeout::TimeoutExt,
    anyhow::{Result},
    std::{
        time::{Instant, Duration},
        sync::{Arc},
        collections::{BTreeMap, BTreeSet},
    },
    uluru::LRUCache as UluruLRU,
};
// these are the kinds of problems faced https://blog.netherlabs.nl/articles/2009/01/18/the-ultimate-so_linger-page-or-why-is-my-tcp-not-reliable
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
pub type Names = Arc<Mutex<UluruLRU<(Nonce, Instant, Duration), { constants::CONGESTION_CONTROL_SIZE }>>>;
#[derive(Clone)]
pub struct CongestionControl(Names);
impl CongestionControl {
    fn new() -> Self {
        Self(Names::default())
    }
    async fn start_timer(&mut self, nw: NarrowWaistPacket) {
        let names_mutex = self.0.clone();
        let mut names_ref = names_mutex.lock().await;
        let nonce = match nw {
            NarrowWaistPacket::Request { nonce, .. } => nonce,
            NarrowWaistPacket::Response{ nonce, .. } => nonce,
        };
        match names_ref.touch(|n|n.0==nonce) {
            true  => {
                let now = Instant::now();
                if let Some(front) = names_ref.front_mut() {
                    front.1 = now;
                }
            },
            false => {
                let now = Instant::now();
                names_ref.insert((nonce, now, Duration::new(1,0)));
            }
        }
    }
    async fn wait(&self, nw: NarrowWaistPacket, rx_mutex: Arc<Mutex<Receiver<InterLinkPacket>>>) -> Option<InterLinkPacket> {
        let names_mutex = self.0.clone();
        let mut names_ref = names_mutex.lock().await;
        let nonce = match nw {
            NarrowWaistPacket::Request { nonce, .. } => nonce,
            NarrowWaistPacket::Response{ nonce, .. } => nonce,
        };
        let ilp = async {
            let mut rx_ref = rx_mutex.lock().await;
            rx_ref.next().await
        };
        let res = names_ref.find(|n|n.0==nonce);
        match res {
            Some(res) => {
                let ilp = ilp.timeout(res.2);
                match ilp.await {
                    Some(Some(ilp)) => {
                        let elapsed = res.1.elapsed();
                        res.2 = elapsed;
                        return Some(ilp)
                    },
                    _ => {
                        res.2 = res.2 * 2;
                        return None
                    }
                }
            },
            None => {
                // what happens when elements are removed from the LRU??? bug?
                return None
            }
        }
    }
}
#[derive(Debug)]
enum AIMD {
    AdditiveIncrease {
        returned: BTreeSet<NWWhereRequestEqResponse>,
        unassociated: BTreeSet<NWWhereRequestEqResponse>
    },
    MultiplicativeDecrease {
        returned: BTreeSet<NWWhereRequestEqResponse>,
        failed: BTreeSet<NWWhereRequestEqResponse>,
        unassociated: BTreeSet<NWWhereRequestEqResponse>
    },
}
#[derive(Clone)]
pub struct TxRx {
    pub link_id: LinkId,
    pub protocol_sid: PrivateIdentityInterface,
    pub p2l_tx: Sender<InterLinkPacket>,
    pub l2p_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
    pub incomplete_responses: Arc<Mutex<WaitMap<HBFIWithoutFrame, BTreeMap<u64, NarrowWaistPacket>>>>,
    pub cc: CongestionControl,
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
            cc: CongestionControl::new(),
            incomplete_responses: Arc::new(Mutex::new(WaitMap::new())),
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
    pub async fn register_hbfi(&mut self, hbfi: HBFI) {
        let incomplete_responses_mutex = self.incomplete_responses.clone();
        let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
        let hbfi2 = HBFIWithoutFrame::new(hbfi);
        incomplete_responses_ref.insert(hbfi2, BTreeMap::new());
    }
    pub async fn next_inbound(self) -> Option<InterLinkPacket> {
        let l2p_rx_mutex = Arc::clone(&self.l2p_rx);
        let mut l2p_rx_ref = l2p_rx_mutex.lock().await;
        l2p_rx_ref.next().await
    }
    async fn send_and_receive(&self, nws: BTreeSet<NWWhereRequestEqResponse>, hbfi_seek: HBFI, rx_mutex: Arc<Mutex<Receiver<InterLinkPacket>>>) -> Result<AIMD> {
        debug!("INITIAL {:?}", nws);
        let total = nws.len();
        let hbfi_seek_no_frame = HBFIWithoutFrame::new(hbfi_seek.clone());
        for nw in nws.clone() {
            let lp = LinkPacket::new(self.link_id.reply_to()?, nw.0.clone());
            let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
            debug!("\t\t|  protocol-to-link");
            let mut p2l_tx = self.p2l_tx.clone();
            match p2l_tx.send(ilp).await {
                Ok(_) => { },
                Err(e) => error!("protocol send error {:?}", e),
            }
        }
        let mut returned: BTreeSet<NWWhereRequestEqResponse> = BTreeSet::new();
        let mut unassociated: BTreeSet<NWWhereRequestEqResponse> = BTreeSet::new();
        let mut counter = 0;
        while counter < total {
            let ilp = async {
                let mut rx_ref = rx_mutex.lock().await;
                rx_ref.next().await
            };
            if let Some(ilp) = ilp.await {
                let nw = ilp.narrow_waist();
                let inbound_hbfi = match nw.clone() {
                    NarrowWaistPacket::Request {..} => { continue },
                    NarrowWaistPacket::Response {hbfi, ..} => { HBFIWithoutFrame::new(hbfi.clone()) },
                };
                if hbfi_seek_no_frame == inbound_hbfi {
                    returned.insert(NWWhereRequestEqResponse::new(nw));
                    counter += 1;
                } else {
                    unassociated.insert(NWWhereRequestEqResponse::new(nw));
                }
            }
        }
        let failed: BTreeSet<NWWhereRequestEqResponse> = nws.difference(&returned).cloned().collect();
        debug!("RETURNED {:?}", returned);
        debug!("FAILED   {:?}", failed);
        debug!("INITIAL == RETURNED: {:?}", (nws == returned) );
        let aimd: AIMD = if failed.len() > 0 {
            AIMD::MultiplicativeDecrease { returned, failed, unassociated }
        } else {
            AIMD::AdditiveIncrease { returned, unassociated }
        };
        Ok(aimd)
    }
    pub async fn unreliable_unordered_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        self.register_hbfi(hbfi_seek.clone()).await;
        let mut congestion_window: u16 = 1;
        let batch_start: u64 = start;
        let batch_end: u64 = start;
        loop {
            let mut nws: BTreeSet<NWWhereRequestEqResponse> = BTreeSet::new();
            for counter in batch_start..=batch_end {
                let hbfi_req = hbfi_seek.clone().offset(counter);
                let nw = NarrowWaistPacket::request(hbfi_req)?;
                nws.insert(NWWhereRequestEqResponse::new(nw));
            }
            let aimd = self.send_and_receive(nws, hbfi_seek.clone(), Arc::clone(&self.unreliable_unordered_response_rx)).await?;
            match aimd {
                AIMD::AdditiveIncrease { returned, unassociated } => {
                    for nw in returned {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { },
                            NarrowWaistPacket::Response { hbfi, offset, .. } => {
                                let incomplete_responses_mutex = self.incomplete_responses.clone();
                                let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
                                if hbfi == hbfi_seek {
                                    if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIWithoutFrame::new(hbfi_seek.clone())) {
                                        let entry = entry.value_mut();
                                        entry.insert(offset, nw.0.clone());
                                        break
                                    };
                                } else {
                                    if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIWithoutFrame::new(hbfi)) {
                                        let entry = entry.value_mut();
                                        entry.insert(offset, nw.0.clone());
                                    };
                                }
                            },
                        }
                    }
                },
                AIMD::MultiplicativeDecrease { returned, failed, unassociated }=> {
                }
            }
            break
        }
        let mut reconstruct: Vec<Vec<u8>> = vec![];
        let incomplete_responses_mutex = self.incomplete_responses.clone();
        let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
        if let Some(map) = incomplete_responses_ref.get(&HBFIWithoutFrame::new(hbfi_seek.clone())) {
            let map = map.value();
            for (_, nw) in map.range(start..=end) {
                let chunk = match hbfi_seek.request_pid {
                    Some(_) => {
                        nw.data(Some(self.protocol_sid.clone()))?
                    },
                    None => {
                        nw.data(None)?
                    },
                };
                reconstruct.push(chunk);
            }
        };
        Ok(reconstruct)
    }
    pub async fn unreliable_sequenced_request(&mut self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        let mut counter = start;
        let mut reconstruct: Vec<Vec<u8>> = vec![];
        while counter <= end {
            let hbfi_req = hbfi.clone().offset(counter);
            let nw = NarrowWaistPacket::request(hbfi_req)?;
            let lp = LinkPacket::new(self.link_id.reply_to()?, nw.clone());
            let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
            debug!("\t\t|  protocol-to-link");
            self.cc.start_timer(nw.clone()).await;
            let mut p2l_tx = self.p2l_tx.clone();
            match p2l_tx.send(ilp).await {
                Ok(_) => { },
                Err(e) => error!("protocol send error {:?}", e),
            }
            let ilp = self.cc.wait(nw.clone(), Arc::clone(&self.unreliable_sequenced_response_rx)).await;
            match ilp {
                Some(ilp) => {
                    let nw = ilp.narrow_waist();
                    match nw.clone() {
                        NarrowWaistPacket::Request { .. } => { },
                        NarrowWaistPacket::Response { hbfi, .. } => {
                            if hbfi.ost < counter {
                                counter = hbfi.ost;
                                continue
                            }
                            let chunk = match hbfi.request_pid {
                                Some(_) => {
                                    nw.data(Some(self.protocol_sid.clone()))?
                                },
                                None => {
                                    nw.data(None)?
                                },
                            };
                            reconstruct.push(chunk);
                        },
                    }
                },
                None => {}
            }
            counter += 1;
        }
        Ok(reconstruct)
    }

    pub async fn reliable_unordered_request(&mut self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        let mut counter = start;
        let mut reconstruct: Vec<Vec<u8>> = vec![];
        while counter <= end {
            let hbfi_req = hbfi.clone().offset(counter);
            let nw = NarrowWaistPacket::request(hbfi_req)?;
            let lp = LinkPacket::new(self.link_id.reply_to()?, nw.clone());
            let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
            debug!("\t\t|  protocol-to-link");
            self.cc.start_timer(nw.clone()).await;
            let mut p2l_tx = self.p2l_tx.clone();
            match p2l_tx.send(ilp).await {
                Ok(_) => { },
                Err(e) => error!("protocol send error {:?}", e),
            }
            let ilp = self.cc.wait(nw.clone(), Arc::clone(&self.reliable_unordered_response_rx)).await;
            match ilp {
                Some(ilp) => {
                    let nw = ilp.narrow_waist();
                    match nw.clone() {
                        NarrowWaistPacket::Request { .. } => { },
                        NarrowWaistPacket::Response { hbfi, .. } => {
                            let chunk = match hbfi.request_pid {
                                Some(_) => {
                                    nw.data(Some(self.protocol_sid.clone()))?
                                },
                                None => {
                                    nw.data(None)?
                                },
                            };
                            reconstruct.push(chunk);
                        },
                    }
                },
                None => {}
            }
            counter += 1;
        }
        Ok(reconstruct)
    }
    pub async fn reliable_ordered_request(&mut self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        let mut counter = start;
        let mut reconstruct: Vec<Vec<u8>> = vec![];
        while counter <= end {
            let hbfi_req = hbfi.clone().offset(counter);
            let nw = NarrowWaistPacket::request(hbfi_req)?;
            let lp = LinkPacket::new(self.link_id.reply_to()?, nw.clone());
            let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
            debug!("\t\t|  protocol-to-link");
            self.cc.start_timer(nw.clone()).await;
            let mut p2l_tx = self.p2l_tx.clone();
            match p2l_tx.send(ilp).await {
                Ok(_) => { },
                Err(e) => error!("protocol send error {:?}", e),
            }
            let ilp = self.cc.wait(nw.clone(), Arc::clone(&self.reliable_ordered_response_rx)).await;
            match ilp {
                Some(ilp) => {
                    let nw = ilp.narrow_waist();
                    match nw.clone() {
                        NarrowWaistPacket::Request { .. } => { },
                        NarrowWaistPacket::Response { hbfi, .. } => {
                            let chunk = match hbfi.request_pid {
                                Some(_) => {
                                    nw.data(Some(self.protocol_sid.clone()))?
                                },
                                None => {
                                    nw.data(None)?
                                },
                            };
                            reconstruct.push(chunk);
                        },
                    }
                },
                None => {}
            }
            counter += 1;
        }
        Ok(reconstruct)
    }
    pub async fn reliable_sequenced_request(&mut self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        let mut counter = start;
        let mut reconstruct: Vec<Vec<u8>> = vec![];
        while counter <= end {
            let hbfi_req = hbfi.clone().offset(counter);
            let nw = NarrowWaistPacket::request(hbfi_req)?;
            let lp = LinkPacket::new(self.link_id.reply_to()?, nw.clone());
            let ilp = InterLinkPacket::new(self.link_id.clone(), lp);
            debug!("\t\t|  protocol-to-link");
            self.cc.start_timer(nw.clone()).await;
            let mut p2l_tx = self.p2l_tx.clone();
            match p2l_tx.send(ilp).await {
                Ok(_) => { },
                Err(e) => error!("protocol send error {:?}", e),
            }
            let ilp = self.cc.wait(nw.clone(), Arc::clone(&self.reliable_sequenced_response_rx)).await;
            match ilp {
                Some(ilp) => {
                    let nw = ilp.narrow_waist();
                    match nw.clone() {
                        NarrowWaistPacket::Request { .. } => { },
                        NarrowWaistPacket::Response { hbfi, .. } => {
                            if hbfi.ost < counter {
                                counter = hbfi.ost;
                                continue
                            }
                            let chunk = match hbfi.request_pid {
                                Some(_) => {
                                    nw.data(Some(self.protocol_sid.clone()))?
                                },
                                None => {
                                    nw.data(None)?
                                },
                            };
                            reconstruct.push(chunk);
                        },
                    }
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
        offset: u64,
        total: u64
    ) -> Result<()> {
        debug!("\t\t|  RESPONSE PACKET FOUND");
        let nw = NarrowWaistPacket::response(self.protocol_sid.clone(), hbfi.clone(), data, offset, total)?;
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
