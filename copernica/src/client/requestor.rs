use {
    crate::{
        narrow_waist::{NarrowWaist, mk_request_packet},
        transport::{TransportPacket, TransportResponse, ReplyTo,
            send_transport_packet, send_transport_response, receive_transport_packet,
        },
        sdri::{Sdri},
        response_store::{Response, ResponseStore, Got},
    },
    std::{
        collections::HashSet,
        sync::{Arc, Mutex},
        time::{Duration},
        thread,
    },
    crossbeam_channel::{
        unbounded,
        Sender,
        Receiver,
        select,
        after,
        never
    },
    log::{trace},
};

fn handle_inbound_packets(whitelist_ref: Arc<Mutex<HashSet<Sdri>>>, rs_ref: Arc<Mutex<ResponseStore>>, transport_packet_receiver: Receiver<TransportPacket>) {
    loop {
        let tp: TransportPacket = transport_packet_receiver.recv().unwrap();
        let packet: NarrowWaist = tp.payload();
        match packet.clone() {
            NarrowWaist::Request { sdri } => {
                trace!("REQUEST ARRIVED: {:?}", sdri);
                continue
            },
            NarrowWaist::Response { sdri, count, total, .. } => {
                //println!("HAZZAH");
                trace!("RESPONSE PACKET ARRIVED: {:?} {}/{}", sdri, count, total-1);
                if whitelist_ref.lock().unwrap().contains(&sdri) {
                    let mut rs_guard = rs_ref.lock().unwrap();
                    rs_guard.insert_packet(packet);
                }
            },
        }
    }
}

#[derive(Clone)]
pub struct CopernicaRequestor {
    remote_addr: ReplyTo,
    listen_addr: ReplyTo,
    whitelist_sdri: Arc<Mutex<HashSet<Sdri>>>,
    transport_packet_receiver: Option<Receiver<TransportPacket>>,
    transport_packet_sender: Option<Sender<TransportPacket>>,
    transport_response_sender: Option<Sender<TransportResponse>>,
    response_store: Arc<Mutex<ResponseStore>>
}

impl CopernicaRequestor {
    pub fn new(listen_addr: String, remote_addr: String) -> CopernicaRequestor {
        let listen_addr = ReplyTo::Udp(listen_addr.parse().unwrap());
        let remote_addr = ReplyTo::Udp(remote_addr.parse().unwrap());
        CopernicaRequestor {
            remote_addr,
            listen_addr,
            transport_packet_receiver: None,
            transport_packet_sender: None,
            transport_response_sender: None,
            whitelist_sdri: Arc::new(Mutex::new(HashSet::new())),
            response_store: Arc::new(Mutex::new(ResponseStore::new(1000))),
        }
    }
    pub fn start_polling(&mut self) {
        let listen_addr_1 = self.listen_addr.clone();
        let listen_addr_2 = self.remote_addr.clone();
        let listen_addr_3 = self.remote_addr.clone();
        let whitelist = self.whitelist_sdri.clone();
        let rs = self.response_store.clone();
        let (inbound_tp_sender, inbound_tp_receiver) = unbounded();
        let (outbound_tr_sender, outbound_tr_receiver) = unbounded();
        let (outbound_tp_sender, outbound_tp_receiver) = unbounded();
        self.transport_packet_receiver = Some(inbound_tp_receiver.clone());
        self.transport_packet_sender = Some(outbound_tp_sender.clone());
        self.transport_response_sender = Some(outbound_tr_sender.clone());
        thread::spawn(move || receive_transport_packet(listen_addr_1, inbound_tp_sender));
        thread::spawn(move || send_transport_response(listen_addr_2, outbound_tr_receiver));
        thread::spawn(move || send_transport_packet(listen_addr_3, outbound_tp_receiver));
        thread::spawn(move || handle_inbound_packets(whitelist, rs, inbound_tp_receiver));
    }

    pub async fn request(&mut self, name: String, retries: u8, timeout_per_retry: u64) -> Option<Response> {
        if let Some(sender) = &self.transport_packet_sender {
            let sdri = Sdri::new(name.clone());
            let sdri2 = sdri.clone();
            let retries2 = retries.clone();
            let timeout_per_retry2 = timeout_per_retry.clone();
            let address = self.listen_addr.clone();
            let rs_guard = self.response_store.clone();
            let mut wl_guard = self.whitelist_sdri.lock().unwrap();
            wl_guard.insert(sdri.clone());
            drop(wl_guard);
            let (overall_completed_s, overall_completed_r) = unbounded();
            let sender2 = sender.clone();
            thread::spawn(move || {
                for x in 0..retries {
                    let packet = TransportPacket::new(address.clone(), mk_request_packet(name.clone()));
                    sender2.send(packet).unwrap();
                    let (completed_s, completed_r) = unbounded();
                    let rs_guard1 = rs_guard.clone();
                    let sdri3 = sdri.clone();
                    thread::spawn(move || {
                        loop {
                            let rs = rs_guard1.lock().unwrap();
                            if rs.complete(&sdri3) {
                                let _ = completed_s.send(true);
                            }
                        }
                    });
                    let duration = Some(Duration::from_millis(timeout_per_retry));
                    let timeout = duration.map(|d| after(d)).unwrap_or_else(never);
                    select! {
                        recv(completed_r) -> _msg => { let _ = overall_completed_s.send(true); trace!("RETRY {}/{} COMPLETED", x, retries); break},
                        recv(timeout) -> _ =>  trace!("RETRY TIMED OUT") ,
                    };
                }
            });
            let duration = Some(Duration::from_millis(timeout_per_retry2 * retries2 as u64 + 10));
            let timeout = duration.map(|d| after(d)).unwrap_or_else(never);
            select! {
                recv(overall_completed_r) -> _msg => trace!("OVERALL COMPLETED"),
                recv(timeout) -> _ =>  trace!("OVERALL TIME OUT") ,
            };
            let rs_ref = self.response_store.clone();
            let rs_guard = rs_ref.lock().unwrap();
            match rs_guard.get(&sdri2).await {
                Some(response) => {
                    match response {
                        Got::Response(response) => return Some(response),
                        Got::NarrowWaist(_) => return None,
                    }
                },
                None => return None,
            }
        }
        None
    }
/*
    pub async fn request2(&mut self, name: String, retries: u8, timeout_per_retry: u64) -> Option<Response> {
        let response_write_ref = self.response_store.clone();
        let response_read_ref  = self.response_store.clone();
        let expected_sdri_p1 = Sdri::new(name.clone());
        let expected_sdri_p2 = expected_sdri_p1.clone();
        if let Some(sender) =  &self.transport_packet_sender {
            let sender = sender.clone();
            let packet = TransportPacket::new(self.listen_addr.clone(), mk_request_packet(name.clone()));
            sender.send(packet).unwrap()
        }

        let (completed_s, completed_r) = unbounded();
        if let Some(receiver) = &self.transport_packet_receiver {
            let receiver = receiver.clone();
            thread::spawn(move || {
                loop {
                    let tp: TransportPacket = receiver.recv().unwrap();
                    let packet: NarrowWaist = tp.payload();
                    match packet.clone() {
                        NarrowWaist::Request { sdri } => {
                            trace!("REQUEST ARRIVED: {:?}", sdri);
                            continue
                        },
                        NarrowWaist::Response { sdri, count, total, .. } => {
                            trace!("RESPONSE PACKET ARRIVED: {:?} {}/{}", sdri, count+1, total);
                            if expected_sdri_p1 == sdri {
                                let mut response_guard = response_write_ref.write().unwrap();
                                response_guard.insert_packet(packet);
                            }
                            if count == total - 1 {
                                completed_s.send(true).unwrap();
                                break
                            }
                        },
                    }
                }
            });
        }
        let duration = Some(Duration::from_millis(timeout_per_retry));
        let timeout = duration.map(|d| after(d)).unwrap_or_else(never);
        select! {
            recv(completed_r) -> _msg => trace!("COMPLETED"),
            recv(timeout) -> _ =>  trace!("TIME OUT") ,
        };
        let response_guard = response_read_ref.read().unwrap();
        match response_guard.get(&expected_sdri_p2).await {
            Some(response) => {
                match response {
                    Got::Response(response) => return Some(response),
                    Got::NarrowWaist(_) => return None,
                }
            },
            None => return None,
        }

    }
    */
}

