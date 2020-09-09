mod file_packing;
mod file_sharing;
mod relay_node;
mod fuse_frontend;
mod fuse;

pub use {
    self::{
        relay_node::{RelayNode},
        file_sharing::{FileSharer},
        file_packing::{Manifest, FileManifest, FilePacker},
        fuse_frontend::{FuseFrontend},
    },
};

use {
    copernica_core::{Copernica, LinkId, ReplyTo, NarrowWaist, WirePacket, InterLinkPacket, HBFI},
    copernica_links::{Link},
    borsh::{BorshSerialize, BorshDeserialize},
    std::{
        thread,
    },
    crossbeam_channel::{Sender},
    sled::{Db, Event},
    anyhow::{Result, anyhow},
};

pub trait CopernicaApp<'a> {
    fn new(db: sled::Db) -> Self;
    fn response_store(&self) -> Db;
    fn get_app_link_tx(&mut self) -> Option<Sender<InterLinkPacket>>;
    fn set_app_link_tx(&mut self, sender: Option<Sender<InterLinkPacket>>);
    fn get_app_link_id(&mut self) -> Option<LinkId>;
    fn set_app_link_id(&mut self, link_id: LinkId);
    #[allow(unreachable_code)]
    fn start(&mut self, mut c: Copernica, ts: Vec<Box<dyn Link>>) -> Result<()> {
        let link_id = LinkId::listen(ReplyTo::Mpsc);
        self.set_app_link_id(link_id.clone());
        let (app_outbound_tx, app_inbound_rx) = c.peer(link_id.clone())?;
        self.set_app_link_tx(Some(app_outbound_tx.clone()));
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
                                let nw = NarrowWaist::try_from_slice(&nw)?;
                                let wp = WirePacket::new(link_id.reply_to(), nw);
                                app_outbound_tx.send(InterLinkPacket::new(ilp.link_id(), wp))?;
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
        let sender = self.get_app_link_tx();
        let link_id = self.get_app_link_id();
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
                        if let Some(link_id) = link_id.clone() {
                            let wp = WirePacket::new(link_id.reply_to(), NarrowWaist::Request{ hbfi: hbfi.clone() });
                            let ilp = InterLinkPacket::new(link_id.clone(), wp);
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
