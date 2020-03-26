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
        sync::{Arc, RwLock},
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

#[derive(Clone)]
pub struct CopernicaRequestor {
    remote_addr: ReplyTo,
    listen_addr: ReplyTo,
    transport_packet_receiver: Option<Receiver<TransportPacket>>,
    transport_packet_sender: Option<Sender<TransportPacket>>,
    transport_response_sender: Option<Sender<TransportResponse>>,
    response_store: Arc<RwLock<ResponseStore>>
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
            response_store: Arc::new(RwLock::new(ResponseStore::new(1000))),
        }
    }
    pub fn start_polling(&mut self) {
        let listen_addr_1 = self.listen_addr.clone();
        let listen_addr_2 = self.remote_addr.clone();
        let listen_addr_3 = self.remote_addr.clone();
        let (inbound_tp_sender, inbound_tp_receiver) = unbounded();
        let (outbound_tr_sender, outbound_tr_receiver) = unbounded();
        let (outbound_tp_sender, outbound_tp_receiver) = unbounded();
        self.transport_packet_receiver = Some(inbound_tp_receiver.clone());
        self.transport_packet_sender = Some(outbound_tp_sender.clone());
        self.transport_response_sender = Some(outbound_tr_sender.clone());
        thread::spawn(move || receive_transport_packet(listen_addr_1, inbound_tp_sender));
        thread::spawn(move || send_transport_response(listen_addr_2, outbound_tr_receiver));
        thread::spawn(move || send_transport_packet(listen_addr_3, outbound_tp_receiver));
    }

    pub async fn request(&mut self, name: String) -> Option<Response> {
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
        }  // end loop
     //   let duration = Some(Duration::from_millis(timeout));
     //   let timeout = duration.map(|d| after(d)).unwrap_or_else(never);
        select! {
            recv(completed_r) -> _msg => {trace!("COMPLETED") },
   //         recv(timeout) -> _ => { println!("TIME OUT") },
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
}

