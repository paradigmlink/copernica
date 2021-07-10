use {
    copernica_common::{
        LinkId, NarrowWaistPacket, NarrowWaistPacketReqEqRes,
        LinkPacket, InterLinkPacket, HBFI, HBFIExcludeFrame,
        PrivateIdentityInterface, PublicIdentity, constants, Operations
    },
    log::{debug, error},
    anyhow::{anyhow, Result},
    std::{
        time::{Duration},
        sync::{mpsc::{sync_channel as channel, Receiver, SyncSender}, Arc, Mutex},
        collections::{BTreeMap, BTreeSet, HashMap},
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
pub enum TxRx {
    Initialized {
        ops: Operations,
        link_id: LinkId,
        protocol_sid: PrivateIdentityInterface,
        p2l_tx: SyncSender<InterLinkPacket>,
        l2p_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
        incomplete_responses: Arc<Mutex<HashMap<HBFIExcludeFrame, BTreeMap<u64, NarrowWaistPacket>>>>,
        unreliable_unordered_response_tx: SyncSender<InterLinkPacket>,
        unreliable_unordered_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
        unreliable_sequenced_response_tx: SyncSender<InterLinkPacket>,
        unreliable_sequenced_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
        reliable_unordered_response_tx: SyncSender<InterLinkPacket>,
        reliable_unordered_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
        reliable_ordered_response_tx: SyncSender<InterLinkPacket>,
        reliable_ordered_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
        reliable_sequenced_response_tx: SyncSender<InterLinkPacket>,
        reliable_sequenced_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
    },
    Inert,
}
impl TxRx {
    pub fn inert() -> TxRx {
        TxRx::Inert
    }
    pub fn init(ops: Operations, link_id: LinkId, protocol_sid: PrivateIdentityInterface, p2l_tx: SyncSender<InterLinkPacket>, l2p_rx: Receiver<InterLinkPacket>) -> TxRx
    {
        let (unreliable_unordered_response_tx, unreliable_unordered_response_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (unreliable_sequenced_response_tx, unreliable_sequenced_response_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (reliable_unordered_response_tx, reliable_unordered_response_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (reliable_ordered_response_tx, reliable_ordered_response_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (reliable_sequenced_response_tx, reliable_sequenced_response_rx) = channel::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        TxRx::Initialized {
            ops,
            link_id,
            protocol_sid,
            p2l_tx,
            l2p_rx: Arc::new(Mutex::new(l2p_rx)),
            incomplete_responses: Arc::new(Mutex::new(HashMap::new())),
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
    pub fn protocol_public_id(&self) -> Result<PublicIdentity> {
        match self {
            TxRx::Initialized { protocol_sid, .. } => {
                Ok(protocol_sid.public_id())
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    fn register_hbfi(&self, hbfi: HBFI) -> Result<()> {
        match self {
            TxRx::Initialized { incomplete_responses, .. } => {
                let incomplete_responses_mutex = incomplete_responses.clone();
                let mut incomplete_responses_ref = incomplete_responses_mutex.lock().unwrap();
                let hbfi2 = HBFIExcludeFrame(hbfi);
                incomplete_responses_ref.insert(hbfi2, BTreeMap::new());
                Ok(())
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn next(self) -> Result<InterLinkPacket> {
        match self {
            TxRx::Initialized { l2p_rx, .. } => {
                let l2p_rx_mutex = Arc::clone(&l2p_rx);
                let l2p_rx_ref = l2p_rx_mutex.lock().unwrap();
                let out = l2p_rx_ref.recv()?;
                Ok(out)
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    fn send_and_receive(&self
        , nws: &BTreeSet<NarrowWaistPacketReqEqRes>
        , hbfi_seek: HBFI
        , rx_mutex: Arc<Mutex<Receiver<InterLinkPacket>>>
        , window_timeout: Duration
        ) -> Result<AIMD> {
        match self {
            TxRx::Initialized { ops, link_id, p2l_tx, .. } => {
                let total = nws.len();
                let hbfi_seek_no_frame = HBFIExcludeFrame(hbfi_seek.clone());
                for nw in nws.clone() {
                    let lp = LinkPacket::new(link_id.reply_to()?, nw.0.clone());
                    let ilp = InterLinkPacket::new(link_id.clone(), lp);
                    debug!("\t\t|  protocol-to-link");
                    ops.protocol_to_link(link_id.link_pid()?, link_id.remote_link_pid()?);
                    let p2l_tx = p2l_tx.clone();
                    match p2l_tx.send(ilp) {
                        Ok(_) => { },
                        Err(e) => error!("protocol send error {:?}", e),
                    }
                }
                struct Data(pub BTreeSet<NarrowWaistPacketReqEqRes>, pub BTreeSet<NarrowWaistPacketReqEqRes>);
                let returned: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
                let unassociated: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
                let data_mutex: Arc<Mutex<Data>> = Arc::new(Mutex::new(Data(returned, unassociated)));
                let data_mutex_to_thread = Arc::clone(&data_mutex);
                let (sender, receiver) = channel(1);
                std::thread::spawn(move || {
                    let mut counter = 0;
                    while counter < total {
                        let rx_ref = rx_mutex.lock().unwrap();
                        match rx_ref.recv() {
                            Ok(ilp) => {
                                let nw = ilp.narrow_waist();
                                let inbound_hbfi = match nw.clone() {
                                    NarrowWaistPacket::Request {..} => {  continue },
                                    NarrowWaistPacket::Response {hbfi, ..} => { HBFIExcludeFrame(hbfi.clone()) },
                                };
                                let mut data = data_mutex_to_thread.lock().unwrap();
                                if hbfi_seek_no_frame == inbound_hbfi {
                                    data.0.insert(NarrowWaistPacketReqEqRes(nw));
                                    counter += 1;
                                } else {
                                    data.1.insert(NarrowWaistPacketReqEqRes(nw));
                                }
                            },
                            Err(_e) => {},
                        }
                    }
                    match sender.send(()) {
                        Ok(_) => {},
                        Err(_) => {},
                    }
                });
                receiver.recv_timeout(window_timeout)?;
                let data = data_mutex.lock().unwrap();
                let returned = data.0.clone();
                let unassociated = data.1.clone();
                let failed: BTreeSet<NarrowWaistPacketReqEqRes> = nws.difference(&returned).cloned().collect();
                let aimd: AIMD = if failed.len() > 0 {
                    AIMD::MultiplicativeDecrease { returned, failed, unassociated }
                } else {
                    AIMD::AdditiveIncrease { returned, unassociated }
                };
                Ok(aimd)
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    fn process_aimd(&self, aimd: AIMD, hbfi_seek: HBFI, congestion_window_size: &mut u64, pending_queue: &mut BTreeSet<NarrowWaistPacketReqEqRes>) {
        match self {
            TxRx::Initialized { incomplete_responses, .. } => {
                match aimd {
                    AIMD::AdditiveIncrease { returned, unassociated } => {
                        let incomplete_responses_mutex = incomplete_responses.clone();
                        let mut incomplete_responses_ref = incomplete_responses_mutex.lock().unwrap();
                        for nw in returned {
                            match nw.clone().0 {
                                NarrowWaistPacket::Request { .. } => { return },
                                NarrowWaistPacket::Response { hbfi, .. } => {
                                    if let Some(entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                                        entry.insert(hbfi.frm.clone(), nw.0.clone());
                                    };
                                },
                            }
                        }
                        for nw in unassociated {
                            match nw.clone().0 {
                                NarrowWaistPacket::Request { .. } => { return },
                                NarrowWaistPacket::Response { hbfi, .. } => {
                                    if let Some(entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                        entry.insert(hbfi.frm.clone(), nw.0.clone());
                                    };
                                },
                            }
                        }
                        *congestion_window_size += 1;
                    },
                    AIMD::MultiplicativeDecrease { returned, failed, unassociated }=> {
                        let incomplete_responses_mutex = incomplete_responses.clone();
                        let mut incomplete_responses_ref = incomplete_responses_mutex.lock().unwrap();
                        for nw in returned {
                            match nw.clone().0 {
                                NarrowWaistPacket::Request { .. } => { return },
                                NarrowWaistPacket::Response { hbfi, .. } => {
                                    if let Some(entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                                        entry.insert(hbfi.frm.clone(), nw.0.clone());
                                    };
                                },
                            }
                        }
                        for nw in unassociated {
                            match nw.clone().0 {
                                NarrowWaistPacket::Request { .. } => { return },
                                NarrowWaistPacket::Response { hbfi, .. } => {
                                    if let Some(entry) = incomplete_responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                        entry.insert(hbfi.frm.clone(), nw.0.clone());
                                    };
                                },
                            }
                        }
                        *congestion_window_size = 1;
                        for nw in failed {
                            pending_queue.insert(nw);
                        }
                    }
                }
            },
            TxRx::Inert => panic!("{}", anyhow!("You must peer with a link first"))
        }
    }
    fn reconstruct_responses(&self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        match self {
            TxRx::Initialized { incomplete_responses, protocol_sid, .. } => {
                let mut reconstruct: Vec<Vec<u8>> = vec![];
                let incomplete_responses_mutex = incomplete_responses.clone();
                let incomplete_responses_ref = incomplete_responses_mutex.lock().unwrap();
                if let Some(map) = incomplete_responses_ref.get(&HBFIExcludeFrame(hbfi_seek.clone())) {
                    for (_, nw) in map.range(start..=end) {
                        let chunk = nw.data(protocol_sid.clone())?;
                        reconstruct.push(chunk);
                    }
                };
                Ok(reconstruct)
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn unreliable_unordered_request(&self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        match self {
            TxRx::Initialized { unreliable_unordered_response_rx, .. } => {
                self.register_hbfi(hbfi_seek.clone())?;
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
                    let aimd = self.send_and_receive(&congestion_window, hbfi_seek.clone(), Arc::clone(&unreliable_unordered_response_rx), window_timeout)?;
                    self.process_aimd(aimd, hbfi_seek.clone(), &mut congestion_window_size, &mut pending_queue);
                }
                let reconstructed = self.reconstruct_responses(hbfi_seek, start, end);
                reconstructed
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn unreliable_sequenced_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        match self {
            TxRx::Initialized { ref unreliable_sequenced_response_rx, .. } => {
                self.register_hbfi(hbfi_seek.clone())?;
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
                    let aimd = self.send_and_receive(&congestion_window, hbfi_seek.clone(), Arc::clone(&unreliable_sequenced_response_rx), window_timeout)?;
                    self.process_aimd(aimd, hbfi_seek.clone(), &mut congestion_window_size, &mut pending_queue);
                }
                let reconstructed = self.reconstruct_responses(hbfi_seek, start, end);
                reconstructed
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }

    pub fn reliable_unordered_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        match self {
            TxRx::Initialized { ref reliable_unordered_response_rx, .. } => {
                self.register_hbfi(hbfi_seek.clone())?;
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
                    let aimd = self.send_and_receive(&congestion_window, hbfi_seek.clone(), Arc::clone(&reliable_unordered_response_rx), window_timeout)?;
                    self.process_aimd(aimd, hbfi_seek.clone(), &mut congestion_window_size, &mut pending_queue);
                }
                let reconstructed = self.reconstruct_responses(hbfi_seek, start, end);
                reconstructed
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn reliable_ordered_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        match self {
            TxRx::Initialized { ref reliable_ordered_response_rx, .. } => {
                self.register_hbfi(hbfi_seek.clone())?;
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
                    let aimd = self.send_and_receive(&congestion_window, hbfi_seek.clone(), Arc::clone(&reliable_ordered_response_rx), window_timeout)?;
                    self.process_aimd(aimd, hbfi_seek.clone(), &mut congestion_window_size, &mut pending_queue);
                }
                let reconstructed = self.reconstruct_responses(hbfi_seek, start, end);
                reconstructed
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn reliable_sequenced_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        match self {
            TxRx::Initialized { ref reliable_sequenced_response_rx, .. } => {
                self.register_hbfi(hbfi_seek.clone())?;
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
                    let aimd = self.send_and_receive(&congestion_window, hbfi_seek.clone(), Arc::clone(&reliable_sequenced_response_rx), window_timeout)?;
                    self.process_aimd(aimd, hbfi_seek.clone(), &mut congestion_window_size, &mut pending_queue);
                }
                let reconstructed = self.reconstruct_responses(hbfi_seek, start, end);
                reconstructed
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn respond(self,
        hbfi: HBFI,
        data: Vec<u8>,
    ) -> Result<()> {
        match self {
            TxRx::Initialized { ref p2l_tx, ref protocol_sid, ref link_id, .. } => {
                debug!("\t\t|  RESPONSE PACKET FOUND");
                let nw = NarrowWaistPacket::response(protocol_sid.clone(), hbfi.clone(), data)?;
                let lp = LinkPacket::new(link_id.reply_to()?, nw);
                let ilp = InterLinkPacket::new(link_id.clone(), lp);
                debug!("\t\t|  protocol-to-link");
                match p2l_tx.send(ilp) {
                    Ok(_) => {},
                    Err(e) => error!("protocol send error {:?}", e),
                }
                Ok(())
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
}
