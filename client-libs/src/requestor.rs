use {
    copernica::{Copernica, Link, ReplyTo, NarrowWaist, WirePacket, InterLinkPacket, HBFI},
    transport::{Transport},
    borsh::{BorshSerialize, BorshDeserialize},
    std::{
        thread,
    },
    crossbeam_channel::{Sender},
    sled::{Db, Event},
    anyhow::{Result, anyhow},
};

pub trait Requestor<'a> {
    fn new(db: sled::Db) -> Self;
    fn response_store(&self) -> Db;
    fn set_sender(&mut self, sender: Option<Sender<InterLinkPacket>>);
    fn get_sender(&mut self) -> Option<Sender<InterLinkPacket>>;
    fn get_link(&mut self) -> Option<Link>;
    fn set_link(&mut self, link: Link);
    #[allow(unreachable_code)]
    fn start(&mut self, mut c: Copernica, ts: Vec<Box<dyn Transport>>) -> Result<()> {
        let link = Link::new(ReplyTo::Mpsc);
        self.set_link(link.clone());
        let (app_outbound_tx, app_inbound_rx) = c.peer(link.clone())?;
        self.set_sender(Some(app_outbound_tx.clone()));
        for t in ts {
            t.run()?;
        }
        let rs = self.response_store();
        c.run(rs.clone())?;
        thread::spawn(move || {
            loop {
                if let Ok(ilp) = app_inbound_rx.recv() {
                    let packet: NarrowWaist = ilp.narrow_waist();
                    match packet.clone() {
                        NarrowWaist::Request { hbfi } => {
                            if let Some(nw) = rs.get(hbfi.try_to_vec()?)? {
                                let wp = WirePacket::new(link.reply_to(), NarrowWaist::try_from_slice(&nw)?);
                                app_outbound_tx.send(InterLinkPacket::new(ilp.link(), wp))?;
                            } else { continue }
                        },
                        NarrowWaist::Response { hbfi, .. } => {
                            rs.insert(hbfi.try_to_vec()?, packet.clone().try_to_vec()?)?;
                        },
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }
    fn get(&mut self, hbfi: HBFI, start: u64, end: u64) -> Result<Vec<u8>> {
        let mut counter = start;
        let mut reconstruct: Vec<u8> = vec![];
        let rs = self.response_store();
        let sender = self.get_sender();
        let link = self.get_link();
        while counter <= end {
            let hbfi = hbfi.clone().offset(counter);
            match rs.get(hbfi.try_to_vec()?)? {
                Some(resp) => {
                    let nw = NarrowWaist::try_from_slice(&resp)?;
                    match nw {
                        NarrowWaist::Request {..} => return Err(anyhow!("Didn't find FileManifest but found a Request")),
                        NarrowWaist::Response {data, ..} => {
                            let (chunk, _) = data.data.split_at(data.len.into());
                            reconstruct.append(&mut chunk.to_vec());
                        }
                    }
                }
                None => {
                    if let Some(sender) = sender.clone() {
                        if let Some(link) = link.clone() {
                            let ilp = InterLinkPacket::new(link.clone(), WirePacket::new(link.reply_to(), NarrowWaist::Request{ hbfi: hbfi.clone() }));
                            let subscriber = rs.watch_prefix(hbfi.try_to_vec()?);
                            sender.send(ilp)?;
                            /*while let Some(event) = (&mut subscriber).await {
                                match event {
                                    Event::Insert{ key: _, value } => {
                                        let nw = NarrowWaist::try_from_slice(&value)?;
                                        match nw {
                                            NarrowWaist::Request {..} => return Err(anyhow!("Didn't find FileManifest but found a Request")),
                                            NarrowWaist::Response {data, ..} => {
                                                let (chunk, _) = data.data.split_at(data.len.into());
                                                reconstruct.append(&mut chunk.to_vec());
                                            }
                                        }
                                    }
                                    Event::Remove {key:_ } => {}
                                }
                            }*/
                            for event in subscriber.take(1) {
                                match event {
                                    Event::Insert{ key: _, value } => {
                                        let nw = NarrowWaist::try_from_slice(&value)?;
                                        match nw {
                                            NarrowWaist::Request {..} => return Err(anyhow!("Didn't find FileManifest but found a Request")),
                                            NarrowWaist::Response {data, ..} => {
                                                let (chunk, _) = data.data.split_at(data.len.into());
                                                reconstruct.append(&mut chunk.to_vec());
                                            }
                                        }
                                    }
                                    Event::Remove {key:_ } => {}
                                }
                            }
                        }
                    }
                }
            }
            counter += 1;
        }
        Ok(reconstruct)
    }
}
