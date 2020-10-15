use {
    copernica_common::{LinkId, NarrowWaistPacket, LinkPacket, InterLinkPacket, HBFI},
    borsh::{BorshSerialize, BorshDeserialize},
    std::{thread},
    crossbeam_channel::{Sender, Receiver, unbounded},
    sled::{Db, Event},
    anyhow::{Result},
};

pub type DropHookFn = Box<dyn Fn() + Send + 'static>;

/*
     s = Service, l = Link, b = Broker, r = Router, 2 = to: e.g. l2b = "link to copernica_broker"
     link::{udp, mpsc_channel, mpsc_corruptor, etc}
                                                            +----------------------------+
    +-----------+s2l_tx   s2l_rx+-----------+l2b_tx   l2b_rx| b2r_tx           b2r_rx    |   +-----------+   +-----------+
    |           +-------------->+           +-------------->-------------------------+   +-->+           +-->+           |
    | Service   |l2s_rx   l2s_tx|   Link    |b2l_rx   b2l_tx| r2b_rx       r2b_tx    |   |   |   Link    |   | Service   |
    |           +<--------------+           +<---------------<-------------------+   |   +<--+           +<--+           |
    +-----------+               +-----------+               |                    |   v   |   +-----------+   +-----------+
                                                            |                +---+---+-+ |
    +-----------+s2l_tx   s2l_rx+-----------+l2b_tx   l2b_rx| b2r_tx   b2r_rx|         | |   +-----------+   +-----------+
    |           +-------------->+           +-------------->---------------->+         | +-->+           +-->+           |
    | Service   |l2s_rx   l2s_tx|   Link    |b2l_rx   b2l_tx| r2b_rx   r2b_tx|  Router | |   |   Link    |   |  Broker   |
    |           +<--------------+           +<---------------<---------------+         | +<--+           +<--+           |
    +-----------+               +-----------+               |                |         | |   +-----------+   +-----------+
                                                            |                +---+---+-+ |
    +-----------+b2l_tx   b2l_rx+-----------+l2b_tx   l2b_rx| b2r_tx      b2r_rx ^   |   |   +-----------+   +-----------+
    |           +-------------->+           +-------------->---------------------+   |   +-->+           +-->+           |
    |  Broker   |l2b_rx   l2b_tx|   Link    |b2l_rx   b2l_tx| r2b_rx          r2b_tx |   |   |   Link    |   | Service   |
    |           +<--------------+           +<---------------<-----------------------+   +<--+           +<--+           |
    +-----------+               +-----------+               |           Broker           |   +-----------+   +-----------+
                                                            +----------------------------+
*/

pub trait Service<'a> {
    fn new(db: sled::Db, drop_hook: DropHookFn) -> Self;
    fn response_store(&self) -> Db;
    fn get_l2s_rx(&mut self) -> Option<Receiver<InterLinkPacket>>;
    fn set_l2s_rx(&mut self, s: Receiver<InterLinkPacket>);
    fn get_s2l_tx(&mut self) -> Option<Sender<InterLinkPacket>>;
    fn set_s2l_tx(&mut self, s: Sender<InterLinkPacket>);
    fn get_link_id(&mut self) -> Option<LinkId>;
    fn set_link_id(&mut self, link_id: LinkId);
    fn handle_narrow_waist(&self, _nw: NarrowWaistPacket) -> Option<NarrowWaistPacket> {
        None
    }
    fn peer(
        &mut self,
        link_id: LinkId,
    ) -> Result<(Sender<InterLinkPacket>, Receiver<InterLinkPacket>)> {
        let (l2s_tx, l2s_rx) = unbounded::<InterLinkPacket>();
        let (s2l_tx, s2l_rx) = unbounded::<InterLinkPacket>();
        self.set_link_id(link_id);
        self.set_s2l_tx(s2l_tx.clone());
        self.set_l2s_rx(l2s_rx.clone());
        Ok((l2s_tx, s2l_rx))
    }
    #[allow(unreachable_code)]
    fn run(&mut self) -> Result<()> {
        let rs = self.response_store();
        let l2s_rx = self.get_l2s_rx();
        let s2l_tx = self.get_s2l_tx();
        let link_id = self.get_link_id();
        thread::spawn(move || {
            if let (Some(l2s_rx), Some(s2l_tx), Some(link_id)) = (l2s_rx, s2l_tx, link_id) {
                loop {
                    if let Ok(ilp) = l2s_rx.recv() {
                        let packet: NarrowWaistPacket = ilp.narrow_waist();
                        match packet.clone() {
                            NarrowWaistPacket::Request { hbfi } => {
                                if let Some(nw) = rs.get(hbfi.try_to_vec()?)? {
                                    let nw = NarrowWaistPacket::try_from_slice(&nw)?;
                                    let wp = LinkPacket::new(link_id.reply_to(), nw);
                                    s2l_tx.send(InterLinkPacket::new(ilp.link_id(), wp))?;
                                } else { continue }
                            },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                rs.insert(hbfi.try_to_vec()?, packet.clone().try_to_vec()?)?;
                            },
                        }
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
        let s2l_tx = self.get_s2l_tx();
        let link_id = self.get_link_id();
        if let Some(s2l_tx) = s2l_tx {
            if let Some(link_id) = link_id {
                while counter <= end {
                    let hbfi = hbfi.clone().offset(counter);
                    match rs.get(hbfi.try_to_vec()?)? {
                        Some(resp) => {
                            let nw = NarrowWaistPacket::try_from_slice(&resp)?;
                            match nw {
                                NarrowWaistPacket::Request {..} => {
                                    match self.handle_narrow_waist(nw) {
                                        Some(nw) => {
                                            let wp = LinkPacket::new(link_id.reply_to(), nw);
                                            let ilp = InterLinkPacket::new(link_id.clone(), wp);
                                            s2l_tx.send(ilp)?;
                                        },
                                        None => {},
                                    }
                                },
                                NarrowWaistPacket::Response {data, ..} => {
                                    let (chunk, _) = data.data.split_at(data.len.into());
                                    reconstruct.append(&mut chunk.to_vec());
                                }
                            }
                        }
                        None => {
                            let wp = LinkPacket::new(link_id.reply_to(), NarrowWaistPacket::Request{ hbfi: hbfi.clone() });
                            let ilp = InterLinkPacket::new(link_id.clone(), wp);
                            let subscriber = rs.watch_prefix(hbfi.try_to_vec()?);
                            s2l_tx.send(ilp)?;
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
                                match event {
                                    Event::Insert{ key: _, value } => {
                                        let nw = NarrowWaistPacket::try_from_slice(&value)?;
                                        match nw {
                                            NarrowWaistPacket::Request {..} => {
                                                match self.handle_narrow_waist(nw) {
                                                    Some(nw) => {
                                                        let wp = LinkPacket::new(link_id.reply_to(), nw);
                                                        let ilp = InterLinkPacket::new(link_id.clone(), wp);
                                                        s2l_tx.send(ilp)?;
                                                    },
                                                    None => {},
                                                }
                                            }
                                            NarrowWaistPacket::Response {data, ..} => {
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
                    counter += 1;
                }
            }
        }
        Ok(reconstruct)
    }
}
