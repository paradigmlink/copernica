use {
    copernica::{Copernica, LinkId, ReplyTo, NarrowWaist, TransportPacket},
    transport::{MpscChannel, Transport},
    borsh::{BorshSerialize, BorshDeserialize},
    std::{
        thread,
    },
    crossbeam_channel::{Sender},
    sled::{Db},
    anyhow::{Result},
    log::{trace},
};

pub trait Requestor<'a> {
    fn new(db: sled::Db) -> Self;
    fn response_store(&self) -> Db;
    fn set_sender(&mut self, sender: Option<Sender<(LinkId, TransportPacket)>>);
    fn set_link_id(&mut self, link_id: Option<LinkId>);
    #[allow(unreachable_code)]
    fn start(&mut self, mut c: Copernica, mut ts: Vec<Box<dyn Transport>>) -> Result<()> {
        let lid = LinkId::new(ReplyTo::Mpsc, 0);
        let mpsc: MpscChannel = Transport::new(lid.clone(), c.create_link(lid.clone())?)?;
        let (sender, receiver) = mpsc.unbounded();
        let sender2 = sender.clone();
        self.set_sender(Some(sender));
        self.set_link_id(Some(lid.clone()));
        ts.push(Box::new(mpsc));
        for t in ts {
            t.run()?;
        }
        let rs = self.response_store();
        c.run(rs.clone())?;
        thread::spawn(move || {
            loop {
                if let Ok((li, tp)) = receiver.recv() {
                    let packet: NarrowWaist = tp.payload();
                    match packet.clone() {
                        NarrowWaist::Request { hbfi } => {
                            trace!("REQUEST ARRIVED: {:?}", hbfi);
                            if let Some(nw) = rs.get(hbfi.try_to_vec()?)? {
                                sender2.send((li, TransportPacket::new(tp.reply_to(), NarrowWaist::try_from_slice(&nw)?)))?;
                            }
                        },
                        NarrowWaist::Response { hbfi, offset, total, .. } => {
                            trace!("RESPONSE PACKET ARRIVED: {:?} {}/{}", hbfi, offset, total-1);
                            rs.insert(hbfi.try_to_vec()?, packet.clone().try_to_vec()?)?;
                        },
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }
}
