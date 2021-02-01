use {
    copernica_common::{LinkId, NarrowWaistPacket, LinkPacket, InterLinkPacket, HBFI, serialization::*},
    copernica_identity::{PrivateIdentity},
    std::{thread},
    log::{debug},
    crossbeam_channel::{Sender, Receiver, unbounded},
    sled::{Db, Event},
    anyhow::{Result},
};

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
pub trait Protocol<'a> {
    fn response_store(&self) -> Db;
    fn get_l2p_rx(&mut self) -> Option<Receiver<InterLinkPacket>>;
    fn set_l2p_rx(&mut self, r: Receiver<InterLinkPacket>);
    fn get_p2l_tx(&mut self) -> Option<Sender<InterLinkPacket>>;
    fn set_p2l_tx(&mut self, s: Sender<InterLinkPacket>);
    fn get_link_id(&mut self) -> Option<LinkId>;
    fn set_link_id(&mut self, link_id: LinkId);
    fn get_response_sid(&mut self) -> PrivateIdentity;
    fn peer_with_link(
        &mut self,
        link_id: LinkId,
    ) -> Result<(Sender<InterLinkPacket>, Receiver<InterLinkPacket>)> {
        let (l2p_tx, l2p_rx) = unbounded::<InterLinkPacket>();
        let (p2l_tx, p2l_rx) = unbounded::<InterLinkPacket>();
        self.set_link_id(link_id);
        self.set_p2l_tx(p2l_tx.clone());
        self.set_l2p_rx(l2p_rx.clone());
        Ok((l2p_tx, p2l_rx))
    }
    #[allow(unreachable_code)]
    fn run(&mut self) -> Result<()> {
        let rs = self.response_store();
        let l2p_rx = self.get_l2p_rx();
        let p2l_tx = self.get_p2l_tx();
        let link_id = self.get_link_id();
        let response_sid = self.get_response_sid();
        thread::spawn(move || {
            if let (Some(l2p_rx), Some(p2l_tx), Some(link_id)) = (l2p_rx, p2l_tx, link_id) {
                loop {
                    if let Ok(ilp) = l2p_rx.recv() {
                        /*
                        debug!("\t\t|  link-to-protocol");
                        let nw: NarrowWaistPacket = ilp.narrow_waist();
                        match nw.clone() {
                            NarrowWaistPacket::Request { hbfi, .. } => {
                                let (_, hbfi_s) = serialize_hbfi(&hbfi)?;
                                if rs.contains_key(hbfi_s.clone())? {
                                    let nw = rs.get(hbfi_s)?;
                                    match nw {
                                        Some(nw) => {
                                            debug!("\t\t|  RESPONSE PACKET FOUND");
                                            let nw = deserialize_narrow_waist_packet(&nw.to_vec())?;
                                            let lp = LinkPacket::new(link_id.reply_to()?, nw);
                                            let ilp = InterLinkPacket::new(link_id.clone(), lp);
                                            debug!("\t\t|  protocol-to-link");
                                            p2l_tx.send(ilp)?;
                                        },
                                        None => {},
                                    }
                                } else {
                                    let hbfi_ctr = hbfi.cleartext_repr();
                                    let (_, hbfi_ctr) = serialize_hbfi(&hbfi_ctr)?;
                                    if rs.contains_key(hbfi_ctr.clone())? {
                                        let nw = rs.get(hbfi_ctr)?;
                                        match nw {
                                            Some(nw) => {
                                                match hbfi.request_pid {
                                                    Some(request_pid) => {
                                                        debug!("\t\t|  RESPONSE PACKET FOUND ENCRYPT IT");
                                                        let nw = deserialize_narrow_waist_packet(&nw.to_vec())?;
                                                        let nw = nw.encrypt_for(request_pid, response_sid.clone())?;
                                                        let lp = LinkPacket::new(link_id.reply_to()?, nw);
                                                        let ilp = InterLinkPacket::new(link_id.clone(), lp);
                                                        debug!("\t\t|  protocol-to-link");
                                                        p2l_tx.send(ilp.clone())?;
                                                    },
                                                    None => {
                                                        debug!("\t\t|  RESPONSE PACKET FOUND CLEARTEXT IT");
                                                        let nw = deserialize_narrow_waist_packet(&nw.to_vec())?;
                                                        let lp = LinkPacket::new(link_id.reply_to()?, nw);
                                                        let ilp = InterLinkPacket::new(link_id.clone(), lp);
                                                        debug!("\t\t|  protocol-to-link");
                                                        p2l_tx.send(ilp)?;
                                                    },
                                                }
                                            },
                                            None => {},
                                        }
                                    };
                                }
                            },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                debug!("\t\t|  RESPONSE PACKET ARRIVED");
                                let (_, hbfi_s) = serialize_hbfi(&hbfi)?;
                                let (_, nw_s) = serialize_narrow_waist_packet(&nw)?;
                                rs.insert(hbfi_s, nw_s)?;
                            },
                        }
                    */
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
        let p2l_tx = self.get_p2l_tx();
        let link_id = self.get_link_id();
        if let (Some(p2l_tx), Some(link_id)) = (p2l_tx, link_id) {
            while counter <= end {
                let hbfi = hbfi.clone().offset(counter);
                let (_, hbfi_s) = serialize_hbfi(&hbfi)?;
                match rs.get(hbfi_s.clone())? {
                    Some(resp) => {
                        let nw = deserialize_narrow_waist_packet(&resp.to_vec())?;
                        let chunk = match hbfi.request_pid {
                            Some(_) => {
                                nw.data(Some(self.get_response_sid()))?
                            },
                            None => {
                                nw.data(None)?
                            },
                        };
                        reconstruct.append(&mut chunk.clone());
                    }
                    None => {
                        let nw = NarrowWaistPacket::request(hbfi.clone())?;
                        let lp = LinkPacket::new(link_id.reply_to()?, nw);
                        let ilp = InterLinkPacket::new(link_id.clone(), lp);
                        let subscriber = rs.watch_prefix(hbfi_s);
                        debug!("\t\t|  protocol-to-link");
                        p2l_tx.send(ilp)?;
                        /*while let Some(event) = (&mut subscriber).await {
                            match event {
                                Event::Insert{ key: _, value } => {
                                    let nw = NarrowWaistPacket::try_from_slice(&value)?;
                                    match nw {
                                        NarrowWaistPacket::Request {..} => return Err(anyhow!("Didn't find FileManifest but found a Request")),
                                        NarrowWaistPacket::Response {data, ..} => {
                                            let (chunk, _) = data.data.split_at(data.len.into());
                                            reconstruct.append(&mut chunk.to_vec());
                                        }
                                    }
                                }
                                Event::Remove {key:_ } => {}
                            }
                        }*/
                        for event in subscriber.take(1) {
                            //debug!("{:?}", event);
                            match event {
                                Event::Insert{ key: _, value } => {
                                    let nw = deserialize_narrow_waist_packet(&value.to_vec())?;
                                    let chunk = match hbfi.request_pid {
                                        Some(_) => {
                                            nw.data(Some(self.get_response_sid()))?
                                        },
                                        None => {
                                            nw.data(None)?
                                        },
                                    };
                                    //debug!("value {:?}", chunk);
                                    reconstruct.append(&mut chunk.clone());
                                }
                                Event::Remove {key:_ } => {}
                            }
                        }
                    }
                }
                counter += 1;
            }
        }
        Ok(reconstruct)
    }
    fn new(db: sled::Db, response_sid: PrivateIdentity) -> Self where Self: Sized; //kept at end cause amp syntax highlighting falls over on the last :
}
