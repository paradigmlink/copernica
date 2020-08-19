use {
    crate::{
        narrow_waist::{NarrowWaist},
        transport::{TransportPacket, ReplyTo,
            send_transport_packet, receive_transport_packet,
        },
        hbfi::{HBFI},
        borsh::{BorshSerialize},
    },
    std::{
        thread,
        path::{
            PathBuf,
        },
    },
    sled::{Db},
    crossbeam_channel::{
        unbounded,
        Sender,
    },
    anyhow::{Result, anyhow},
    log::{trace},
};

pub trait Requestor {
    fn new(db: sled::Db, inbound: ReplyTo, outbound: ReplyTo) -> Self;
    fn inbound(&self) -> ReplyTo;
    fn outbound(&self) -> ReplyTo;
    fn response_store(&self) -> Db;
    fn get_sender(&self) -> Option<Sender<TransportPacket>>;
    fn set_sender(&mut self, sender: Option<Sender<TransportPacket>>);
    fn start_polling(&mut self) -> Result<()> {
        let inbound = self.inbound();
        let outbound = self.outbound();
        let rs = self.response_store();
        let (inbound_tp_sender, inbound_tp_receiver) = unbounded();
        let (outbound_tp_sender, outbound_tp_receiver) = unbounded();
        let transport_packet_receiver = inbound_tp_receiver.clone();
        self.set_sender(Some(outbound_tp_sender.clone()));
        thread::spawn(move || receive_transport_packet(inbound, inbound_tp_sender));
        thread::spawn(move || send_transport_packet(outbound, outbound_tp_receiver));
        thread::spawn(move || {
            loop {
                let tp: TransportPacket = transport_packet_receiver.recv().unwrap();
                let packet: NarrowWaist = tp.payload();
                match packet.clone() {
                    NarrowWaist::Request { hbfi } => {
                        trace!("REQUEST ARRIVED: {:?}", hbfi);
                        continue
                    },
                    NarrowWaist::Response { hbfi, offset, total, .. } => {
                        println!("HAZZAH");
                        trace!("RESPONSE PACKET ARRIVED: {:?} {}/{}", hbfi, offset, total-1);
                        rs.insert(hbfi.try_to_vec()?, packet.clone().try_to_vec()?)?;
                    },
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }

    fn request(&mut self, _hbfi: HBFI) -> Result<NarrowWaist> {
        if let Some(_sender) = self.get_sender() {
        }
        Err(anyhow!("Error: Transport Packet Sender not initialized"))
    }
}
