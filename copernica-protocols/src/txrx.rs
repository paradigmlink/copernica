use {
    copernica_common::{LinkId, NarrowWaistPacket, NarrowWaistPacketReqEqRes, LinkPacket, InterLinkPacket, HBFI, HBFIExcludeFrame, PrivateIdentityInterface,
    constants},
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
        time::{Duration},
        sync::{Arc},
        collections::{BTreeMap, BTreeSet},
    },
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
#[derive(Debug)]
enum AIMD {
    AdditiveIncrease {
        returned: BTreeSet<NarrowWaistPacketReqEqRes>,
        unassociated: BTreeSet<NarrowWaistPacketReqEqRes>
    },
    MultiplicativeDecrease {
        returned: BTreeSet<NarrowWaistPacketReqEqRes>,
        failed: BTreeSet<NarrowWaistPacketReqEqRes>,
        unassociated: BTreeSet<NarrowWaistPacketReqEqRes>
    },
}
#[derive(Clone)]
pub struct TxRx {
    pub link_id: LinkId,
    pub protocol_sid: PrivateIdentityInterface,
    pub p2l_tx: Sender<InterLinkPacket>,
    pub l2p_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
    pub incomplete_responses: Arc<Mutex<WaitMap<HBFIExcludeFrame, BTreeMap<u64, NarrowWaistPacket>>>>,
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
        let hbfi2 = HBFIExcludeFrame(hbfi);
        incomplete_responses_ref.insert(hbfi2, BTreeMap::new());
    }
    pub async fn next_inbound(self) -> Option<InterLinkPacket> {
        let l2p_rx_mutex = Arc::clone(&self.l2p_rx);
        let mut l2p_rx_ref = l2p_rx_mutex.lock().await;
        l2p_rx_ref.next().await
    }
    async fn send_and_receive(&self
        , nws: &BTreeSet<NarrowWaistPacketReqEqRes>
        , hbfi_seek: HBFI
        , rx_mutex: Arc<Mutex<Receiver<InterLinkPacket>>>
        , window_timeout: Duration
        ) -> Result<AIMD> {
        let total = nws.len();
        let hbfi_seek_no_frame = HBFIExcludeFrame(hbfi_seek.clone());
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
        let mut returned: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
        let mut unassociated: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
        let window = async {
            let mut counter = 0;
            while counter < total {
                let ilp = async {
                    let mut rx_ref = rx_mutex.lock().await;
                    rx_ref.next().await
                };
                if let Some(ilp) = ilp.await {
                    let nw = ilp.narrow_waist();
                    let inbound_hbfi = match nw.clone() {
                        NarrowWaistPacket::Request {..} => {  continue },
                        NarrowWaistPacket::Response {hbfi, ..} => { HBFIExcludeFrame(hbfi.clone()) },
                    };
                    if hbfi_seek_no_frame == inbound_hbfi {
                        returned.insert(NarrowWaistPacketReqEqRes(nw));
                        counter += 1;
                    } else {
                        unassociated.insert(NarrowWaistPacketReqEqRes(nw));
                    }
                }
            }
        };
        let window_timeout = window.timeout(window_timeout);
        window_timeout.await;
        let failed: BTreeSet<NarrowWaistPacketReqEqRes> = nws.difference(&returned).cloned().collect();
        let aimd: AIMD = if failed.len() > 0 {
            AIMD::MultiplicativeDecrease { returned, failed, unassociated }
        } else {
            AIMD::AdditiveIncrease { returned, unassociated }
        };
        Ok(aimd)
    }
    pub async fn unreliable_unordered_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        self.register_hbfi(hbfi_seek.clone()).await;
        let window_timeout = Duration::new(1,0);
        let mut pending_queue: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
        let mut congestion_window: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
        let mut congestion_window_size: u64 = 1;
        for counter in start..=end {
            let hbfi_req = hbfi_seek.clone().offset(counter);
            let nw = NarrowWaistPacket::request(hbfi_req)?;
            pending_queue.insert(NarrowWaistPacketReqEqRes(nw));
        }
        loop {
            if pending_queue.len() == 0 { break }
            congestion_window.clear();
            for _ in 0..congestion_window_size {
                match pending_queue.pop_first() {
                    Some(nw) => {
                        congestion_window.insert(nw);
                    },
                    None => continue,
                }
            }
            let aimd = self.send_and_receive(&congestion_window, hbfi_seek.clone(), Arc::clone(&self.unreliable_unordered_response_rx), window_timeout).await?;
            match aimd {
                AIMD::AdditiveIncrease { returned, unassociated } => {
                    let incomplete_responses_mutex = self.incomplete_responses.clone();
                    let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
                    for nw in returned {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    for nw in unassociated {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    congestion_window_size += 1;
                },
                AIMD::MultiplicativeDecrease { returned, failed, unassociated }=> {
                    let incomplete_responses_mutex = self.incomplete_responses.clone();
                    let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
                    for nw in returned {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    for nw in unassociated {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    congestion_window_size = 1;
                    for nw in failed {
                        pending_queue.insert(nw);
                    }
                }
            }
        }
        let mut reconstruct: Vec<Vec<u8>> = vec![];
        let incomplete_responses_mutex = self.incomplete_responses.clone();
        let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
        if let Some(map) = incomplete_responses_ref.get(&HBFIExcludeFrame(hbfi_seek.clone())) {
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
    pub async fn unreliable_sequenced_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        self.register_hbfi(hbfi_seek.clone()).await;
        let window_timeout = Duration::new(1,0);
        let mut pending_queue: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
        let mut congestion_window: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
        let mut congestion_window_size: u64 = 1;
        for counter in start..=end {
            let hbfi_req = hbfi_seek.clone().offset(counter);
            let nw = NarrowWaistPacket::request(hbfi_req)?;
            pending_queue.insert(NarrowWaistPacketReqEqRes(nw));
        }
        loop {
            if pending_queue.len() == 0 { break }
            congestion_window.clear();
            for _ in 0..congestion_window_size {
                match pending_queue.pop_first() {
                    Some(nw) => {
                        congestion_window.insert(nw);
                    },
                    None => continue,
                }
            }
            let aimd = self.send_and_receive(&congestion_window, hbfi_seek.clone(), Arc::clone(&self.unreliable_sequenced_response_rx), window_timeout).await?;
            match aimd {
                AIMD::AdditiveIncrease { returned, unassociated } => {
                    let incomplete_responses_mutex = self.incomplete_responses.clone();
                    let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
                    for nw in returned {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    for nw in unassociated {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    congestion_window_size += 1;
                },
                AIMD::MultiplicativeDecrease { returned, failed, unassociated }=> {
                    let incomplete_responses_mutex = self.incomplete_responses.clone();
                    let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
                    for nw in returned {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    for nw in unassociated {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    congestion_window_size = 1;
                    for nw in failed {
                        pending_queue.insert(nw);
                    }
                }
            }
        }
        let mut reconstruct: Vec<Vec<u8>> = vec![];
        let incomplete_responses_mutex = self.incomplete_responses.clone();
        let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
        if let Some(map) = incomplete_responses_ref.get(&HBFIExcludeFrame(hbfi_seek.clone())) {
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

    pub async fn reliable_unordered_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        self.register_hbfi(hbfi_seek.clone()).await;
        let window_timeout = Duration::new(1,0);
        let mut pending_queue: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
        let mut congestion_window: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
        let mut congestion_window_size: u64 = 1;
        for counter in start..=end {
            let hbfi_req = hbfi_seek.clone().offset(counter);
            let nw = NarrowWaistPacket::request(hbfi_req)?;
            pending_queue.insert(NarrowWaistPacketReqEqRes(nw));
        }
        loop {
            if pending_queue.len() == 0 { break }
            congestion_window.clear();
            for _ in 0..congestion_window_size {
                match pending_queue.pop_first() {
                    Some(nw) => {
                        congestion_window.insert(nw);
                    },
                    None => continue,
                }
            }
            let aimd = self.send_and_receive(&congestion_window, hbfi_seek.clone(), Arc::clone(&self.reliable_unordered_response_rx), window_timeout).await?;
            match aimd {
                AIMD::AdditiveIncrease { returned, unassociated } => {
                    let incomplete_responses_mutex = self.incomplete_responses.clone();
                    let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
                    for nw in returned {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    for nw in unassociated {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    congestion_window_size += 1;
                },
                AIMD::MultiplicativeDecrease { returned, failed, unassociated }=> {
                    let incomplete_responses_mutex = self.incomplete_responses.clone();
                    let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
                    for nw in returned {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    for nw in unassociated {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    congestion_window_size = 1;
                    for nw in failed {
                        pending_queue.insert(nw);
                    }
                }
            }
        }
        let mut reconstruct: Vec<Vec<u8>> = vec![];
        let incomplete_responses_mutex = self.incomplete_responses.clone();
        let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
        if let Some(map) = incomplete_responses_ref.get(&HBFIExcludeFrame(hbfi_seek.clone())) {
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
    pub async fn reliable_ordered_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        self.register_hbfi(hbfi_seek.clone()).await;
        let window_timeout = Duration::new(1,0);
        let mut pending_queue: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
        let mut congestion_window: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
        let mut congestion_window_size: u64 = 1;
        for counter in start..=end {
            let hbfi_req = hbfi_seek.clone().offset(counter);
            let nw = NarrowWaistPacket::request(hbfi_req)?;
            pending_queue.insert(NarrowWaistPacketReqEqRes(nw));
        }
        loop {
            if pending_queue.len() == 0 { break }
            congestion_window.clear();
            for _ in 0..congestion_window_size {
                match pending_queue.pop_first() {
                    Some(nw) => {
                        congestion_window.insert(nw);
                    },
                    None => continue,
                }
            }
            let aimd = self.send_and_receive(&congestion_window, hbfi_seek.clone(), Arc::clone(&self.reliable_ordered_response_rx), window_timeout).await?;
            match aimd {
                AIMD::AdditiveIncrease { returned, unassociated } => {
                    let incomplete_responses_mutex = self.incomplete_responses.clone();
                    let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
                    for nw in returned {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    for nw in unassociated {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    congestion_window_size += 1;
                },
                AIMD::MultiplicativeDecrease { returned, failed, unassociated }=> {
                    let incomplete_responses_mutex = self.incomplete_responses.clone();
                    let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
                    for nw in returned {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    for nw in unassociated {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    congestion_window_size = 1;
                    for nw in failed {
                        pending_queue.insert(nw);
                    }
                }
            }
        }
        let mut reconstruct: Vec<Vec<u8>> = vec![];
        let incomplete_responses_mutex = self.incomplete_responses.clone();
        let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
        if let Some(map) = incomplete_responses_ref.get(&HBFIExcludeFrame(hbfi_seek.clone())) {
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
    pub async fn reliable_sequenced_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        self.register_hbfi(hbfi_seek.clone()).await;
        let window_timeout = Duration::new(1,0);
        let mut pending_queue: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
        let mut congestion_window: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
        let mut congestion_window_size: u64 = 1;
        for counter in start..=end {
            let hbfi_req = hbfi_seek.clone().offset(counter);
            let nw = NarrowWaistPacket::request(hbfi_req)?;
            pending_queue.insert(NarrowWaistPacketReqEqRes(nw));
        }
        loop {
            if pending_queue.len() == 0 { break }
            congestion_window.clear();
            for _ in 0..congestion_window_size {
                match pending_queue.pop_first() {
                    Some(nw) => {
                        congestion_window.insert(nw);
                    },
                    None => continue,
                }
            }
            let aimd = self.send_and_receive(&congestion_window, hbfi_seek.clone(), Arc::clone(&self.reliable_sequenced_response_rx), window_timeout).await?;
            match aimd {
                AIMD::AdditiveIncrease { returned, unassociated } => {
                    let incomplete_responses_mutex = self.incomplete_responses.clone();
                    let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
                    for nw in returned {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    for nw in unassociated {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    congestion_window_size += 1;
                },
                AIMD::MultiplicativeDecrease { returned, failed, unassociated }=> {
                    let incomplete_responses_mutex = self.incomplete_responses.clone();
                    let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
                    for nw in returned {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    for nw in unassociated {
                        match nw.clone().0 {
                            NarrowWaistPacket::Request { .. } => { continue },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                if let Some(mut entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                    let entry = entry.value_mut();
                                    entry.insert(hbfi.frm.clone(), nw.0.clone());
                                };
                            },
                        }
                    }
                    congestion_window_size = 1;
                    for nw in failed {
                        pending_queue.insert(nw);
                    }
                }
            }
        }
        let mut reconstruct: Vec<Vec<u8>> = vec![];
        let incomplete_responses_mutex = self.incomplete_responses.clone();
        let incomplete_responses_ref = incomplete_responses_mutex.lock().await;
        if let Some(map) = incomplete_responses_ref.get(&HBFIExcludeFrame(hbfi_seek.clone())) {
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
    pub async fn respond(mut self,
        hbfi: HBFI,
        data: Vec<u8>,
    ) -> Result<()> {
        debug!("\t\t|  RESPONSE PACKET FOUND");
        let nw = NarrowWaistPacket::response(self.protocol_sid.clone(), hbfi.clone(), data)?;
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
