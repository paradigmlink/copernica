use {
    copernica_common::{
        LinkId, NarrowWaistPacket, NarrowWaistPacketReqEqRes,
        LinkPacket, InterLinkPacket, HBFI, HBFIExcludeFrame,
        PrivateIdentityInterface, PublicIdentity, constants, Operations,
    },
    log::{trace,
        debug,
        error
    },
    anyhow::{anyhow, Result},
    crossbeam_channel::{Receiver, Sender, bounded, unbounded, RecvTimeoutError, SendError},
    std::{
        time::{Duration},
        sync::{Arc, Mutex},
        collections::{BTreeSet, HashMap},
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
enum Reliability {
    UnreliableSequenced,
    ReliableSequenced,
    ReliableOrdered,
}
#[derive(Debug)]
enum AIMD {
    AdditiveIncrease,
    MultiplicativeDecrease {
        failed: BTreeSet<NarrowWaistPacketReqEqRes>,
    },
}
#[derive(Clone)]
pub enum TxRx {
    Initialized {
        label: String,
        ops: Operations,
        link_id: LinkId,
        protocol_sid: PrivateIdentityInterface,
        p2l_tx: Sender<InterLinkPacket>,
        l2p_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
        responses: Arc<Mutex<HashMap<HBFIExcludeFrame, BTreeSet<NarrowWaistPacketReqEqRes>>>>,
        unreliable_sequenced_response_tx: Sender<InterLinkPacket>,
        unreliable_sequenced_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
        reliable_sequenced_response_tx: Sender<InterLinkPacket>,
        reliable_sequenced_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
        reliable_ordered_response_tx: Sender<InterLinkPacket>,
        reliable_ordered_response_rx: Arc<Mutex<Receiver<InterLinkPacket>>>,
    },
    Inert,
}
impl TxRx {
    pub fn inert() -> TxRx {
        TxRx::Inert
    }
    pub fn init(label: String, ops: Operations, link_id: LinkId, protocol_sid: PrivateIdentityInterface, p2l_tx: Sender<InterLinkPacket>, l2p_rx: Receiver<InterLinkPacket>) -> TxRx
    {
        let (unreliable_sequenced_response_tx, unreliable_sequenced_response_rx) = bounded::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (reliable_sequenced_response_tx, reliable_sequenced_response_rx) = bounded::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        let (reliable_ordered_response_tx, reliable_ordered_response_rx) = bounded::<InterLinkPacket>(constants::BOUNDED_BUFFER_SIZE);
        TxRx::Initialized {
            label,
            ops,
            link_id,
            protocol_sid,
            p2l_tx,
            l2p_rx: Arc::new(Mutex::new(l2p_rx)),
            responses: Arc::new(Mutex::new(HashMap::new())),
            unreliable_sequenced_response_rx: Arc::new(Mutex::new(unreliable_sequenced_response_rx)),
            unreliable_sequenced_response_tx,
            reliable_sequenced_response_rx: Arc::new(Mutex::new(reliable_sequenced_response_rx)),
            reliable_sequenced_response_tx,
            reliable_ordered_response_rx: Arc::new(Mutex::new(reliable_ordered_response_rx)),
            reliable_ordered_response_tx,
         }
    }
    fn label(&self) -> Result<String> {
        match self {
            TxRx::Initialized { label, .. } => {
                Ok(label.clone())
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
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
            TxRx::Initialized { responses, .. } => {
                let responses_mutex = responses.clone();
                let mut responses_ref = responses_mutex.lock().unwrap();
                responses_ref.insert(HBFIExcludeFrame(hbfi), BTreeSet::new());
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
    fn reconstruct_responses(&self, hbfi_seek: HBFI, start: u64, end: u64) -> Result<Vec<Vec<u8>>> {
        match self {
            TxRx::Initialized { responses, protocol_sid, .. } => {
                let mut reconstruct: Vec<Vec<u8>> = vec![];
                let responses_mutex = responses.clone();
                let responses_ref = responses_mutex.lock().unwrap();
                //debug!("{:#?}", responses_ref);
                if let Some(set) = responses_ref.get(&HBFIExcludeFrame(hbfi_seek.clone())) {
                    let start_nw = NarrowWaistPacket::request(hbfi_seek.clone().offset(start))?;
                    let end_nw = NarrowWaistPacket::request(hbfi_seek.clone().offset(end))?;
                    for nw in set.range(&NarrowWaistPacketReqEqRes(start_nw)..=&NarrowWaistPacketReqEqRes(end_nw)) {
                        let chunk = nw.0.data(protocol_sid.clone())?;
                        reconstruct.push(chunk);
                    }
                };
                Ok(reconstruct)
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    fn process_aimd(&self, aimd: AIMD, reliability: &Reliability, congestion_window_size: &mut u64, pending_queue: &mut BTreeSet<NarrowWaistPacketReqEqRes>) {
        match self {
            TxRx::Initialized { .. } => {
                match aimd {
                    AIMD::AdditiveIncrease => {
                        *congestion_window_size += 1;
                    },
                    AIMD::MultiplicativeDecrease { failed }=> {
                        match reliability {
                            Reliability::ReliableOrdered => {
                                *congestion_window_size = 1;
                                for nw in failed {
                                    pending_queue.insert(nw);
                                }
                            },
                            Reliability::ReliableSequenced => {
                                *congestion_window_size = 1;
                                for nw in failed {
                                    pending_queue.insert(nw);
                                }
                            },
                            Reliability::UnreliableSequenced => {
                                *congestion_window_size = 1;
                                /*
                                match *congestion_window_size / 2 {
                                    0 => *congestion_window_size = 1,
                                    i => *congestion_window_size = i,
                                }
                                */
                            },
                        }
                    }
                }
            },
            TxRx::Inert => panic!("{}", anyhow!("You must peer with a link first"))
        }
    }
    fn send_and_receive(&self
        , congestion_window: Arc<Mutex<BTreeSet<NarrowWaistPacketReqEqRes>>>
        , hbfi_seek: HBFI
        , reliability: &Reliability
        , rx_mutex: Arc<Mutex<Receiver<InterLinkPacket>>>
        , retries: &mut u64
        , window_timeout: &mut u64
        , (timeout_tx, timeout_rx): (Sender<()>, Receiver<()>)
        ) -> Result<AIMD> {
        match self {
            TxRx::Initialized { ops, link_id, p2l_tx, responses, .. } => {
                let congestion_window_guard = Arc::clone(&congestion_window);
                let congestion_window_ref = congestion_window_guard.lock().unwrap();
                for nw in congestion_window_ref.iter() {
                    let lp = LinkPacket::new(link_id.reply_to()?, nw.0.clone());
                    let ilp = InterLinkPacket::new(link_id.clone(), lp);
                    trace!("\t\t|  protocol-to-link");
                    ops.message_from(self.label()?);
                    let p2l_tx = p2l_tx.clone();
                    match p2l_tx.send(ilp.clone()) {
                        Ok(_) => {
                            //debug!("SENT ILP {:#?}", ilp);
                        },
                        Err(e) => error!("protocol send error {:?}", e),
                    }
                }
                drop(congestion_window_ref);
                let responses_to_thread = Arc::clone(&responses);
                let congestion_window_to_thread = Arc::clone(&congestion_window);
                std::thread::spawn(move || {
                    loop {
                        let rx_ref = rx_mutex.lock().unwrap();
                        match rx_ref.recv() {
                            Ok(ilp) => {
                                let nw = ilp.narrow_waist();
                                let responses_mutex = Arc::clone(&responses_to_thread);
                                let mut responses_ref = responses_mutex.lock().unwrap();
                                match nw.clone() {
                                    NarrowWaistPacket::Request { .. } => { continue },
                                    NarrowWaistPacket::Response { hbfi, .. } => {
                                        if let Some(entry) = responses_ref.get_mut(&HBFIExcludeFrame(hbfi.clone())) {
                                            entry.insert(NarrowWaistPacketReqEqRes(nw.clone()));
                                            let congestion_window_ref = congestion_window_to_thread.lock().unwrap();
                                            let outstanding: BTreeSet<NarrowWaistPacketReqEqRes> = congestion_window_ref.difference(&entry).cloned().collect();
                                            //debug!("\nCONGESTION {:#?}\nRETURNED {:#?}\nOUTSTANDING {:#?}", congestion_window_ref, nw, outstanding);
                                            if outstanding.is_empty() {
                                                match timeout_tx.send(()) {
                                                    Ok(_) => {},
                                                    Err(SendError(_)) => { debug!("TxRx timeout mechanism is broken <- this error needs fixing") },
                                                }
                                                break
                                            }
                                        };
                                    },
                                }
                            },
                            Err(e) => {debug!("{}", e)},
                        }
                        drop(rx_ref);
                    }
                });
                let out = timeout_rx.recv_timeout(Duration::from_millis(*window_timeout));
                match out {
                    Err(RecvTimeoutError::Timeout) => *retries -= &1,
                    Err(RecvTimeoutError::Disconnected) => { debug!("TxRx.send_and_receive receiver has disconnected"); *retries -= &1 },
                    Ok(()) => {},
                }
                let mut responses_ref = responses.lock().unwrap();
                if let Some(entry) = responses_ref.get_mut(&HBFIExcludeFrame(hbfi_seek.clone())) {
                    let congestion_window_guard = Arc::clone(&congestion_window);
                    let congestion_window_ref = congestion_window_guard.lock().unwrap();
                    let failed: BTreeSet<NarrowWaistPacketReqEqRes> = congestion_window_ref.difference(&entry).cloned().collect();
                    let aimd: AIMD = if failed.len() > 0 {
                        match reliability {
                            Reliability::ReliableOrdered => {
                                AIMD::MultiplicativeDecrease { failed }
                            },
                            Reliability::ReliableSequenced => {
                                AIMD::MultiplicativeDecrease { failed }
                            },
                            Reliability::UnreliableSequenced => {
                                AIMD::MultiplicativeDecrease { failed: BTreeSet::new() }
                            },
                        }
                    } else {
                        AIMD::AdditiveIncrease
                    };
                    Ok(aimd)
                } else {
                    Err(anyhow!("hbfi not present in responses"))
                }
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    fn request(&mut self
        , reliability: &Reliability
        , rx_mutex: Arc<Mutex<Receiver<InterLinkPacket>>>
        , hbfi_seek: HBFI
        , start: u64
        , end: u64
        , retries: &mut u64
        , window_timeout: &mut u64) -> Result<Vec<Vec<u8>>> {
            self.register_hbfi(hbfi_seek.clone())?;
            let mut pending_queue: BTreeSet<NarrowWaistPacketReqEqRes> = BTreeSet::new();
            let congestion_window: Arc<Mutex<BTreeSet<NarrowWaistPacketReqEqRes>>> = Arc::new(Mutex::new(BTreeSet::new()));
            let mut congestion_window_size: u64 = 1;
            for counter in start..=end {
                let hbfi_req = hbfi_seek.clone().offset(counter);
                let nw = NarrowWaistPacket::request(hbfi_req)?;
                pending_queue.insert(NarrowWaistPacketReqEqRes(nw));
            }
            let timeout_txrx = unbounded::<()>();
            loop {
                if retries <= &mut 0 { break }
                if pending_queue.len() <= 0 { break }
                let congestion_window_guard = Arc::clone(&congestion_window);
                let mut congestion_window_ref = congestion_window_guard.lock().unwrap();
                congestion_window_ref.clear();
                for _ in 0..congestion_window_size {
                    match pending_queue.pop_first() {
                        Some(nw) => {
                            congestion_window_ref.insert(nw);
                        },
                        None => continue,
                    }
                }
                drop(congestion_window_ref);
                let aimd = self.send_and_receive(Arc::clone(&congestion_window), hbfi_seek.clone(), &reliability, rx_mutex.clone(), retries, window_timeout, timeout_txrx.clone())?;
                self.process_aimd(aimd, &reliability, &mut congestion_window_size, &mut pending_queue);
            }
            self.reconstruct_responses(hbfi_seek, start, end)
    }
    pub fn unreliable_sequenced_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64, retries: &mut u64) -> Result<Vec<Vec<u8>>> {
        match self {
            TxRx::Initialized { ref unreliable_sequenced_response_rx, .. } => {
                let mut window_timeout = 300;
                let rx_mutex = Arc::clone(&unreliable_sequenced_response_rx);
                self.request(
                    &Reliability::UnreliableSequenced,
                    rx_mutex,
                    hbfi_seek,
                    start,
                    end,
                    retries,
                    &mut window_timeout
                )
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn reliable_sequenced_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64, retries: &mut u64) -> Result<Vec<Vec<u8>>> {
        match self {
            TxRx::Initialized { ref reliable_sequenced_response_rx, .. } => {
                let mut window_timeout = 300;
                let rx_mutex = Arc::clone(&reliable_sequenced_response_rx);
                self.request(
                    &Reliability::ReliableSequenced,
                    rx_mutex,
                    hbfi_seek,
                    start,
                    end,
                    retries,
                    &mut window_timeout
                )
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn reliable_ordered_request(&mut self, hbfi_seek: HBFI, start: u64, end: u64, retries: &mut u64) -> Result<Vec<Vec<u8>>> {
        match self {
            TxRx::Initialized { ref reliable_ordered_response_rx, .. } => {
                let mut window_timeout = 300;
                let rx_mutex = Arc::clone(&reliable_ordered_response_rx);
                self.request(
                    &Reliability::ReliableOrdered,
                    rx_mutex,
                    hbfi_seek,
                    start,
                    end,
                    retries,
                    &mut window_timeout
                )
            },
            TxRx::Inert => Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn respond(self,
        hbfi: HBFI,
        data: Vec<u8>,
    ) -> Result<()> {
        match self {
            TxRx::Initialized { ref p2l_tx, ref protocol_sid, ref link_id, ref ops, ref label, .. } => {
                trace!("\t\t|  RESPONSE PACKET FOUND");
                ops.found_response_upstream(label.clone());
                let nw = NarrowWaistPacket::response(protocol_sid.clone(), hbfi.clone(), data)?;
                let lp = LinkPacket::new(link_id.reply_to()?, nw);
                let ilp = InterLinkPacket::new(link_id.clone(), lp);
                trace!("\t\t|  protocol-to-link");
                ops.message_from(label.clone());
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
