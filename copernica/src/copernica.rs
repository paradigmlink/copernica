use {
    crate::{
        router::{Router},
        channel::{LinkId},
        packets::{TransportPacket},
        link::{Link},
    },
    crossbeam_channel::{Sender, Receiver, unbounded},
    std::{
        collections::{HashMap},
    },
    anyhow::{anyhow, Result},
};

#[derive(Clone)]
pub struct Copernica {
    inbound_tx: Sender<(LinkId, TransportPacket)>,
    inbound_rx: Receiver<(LinkId, TransportPacket)>,
    outbound_tx: Sender<(LinkId, TransportPacket)>,
    outbound_rx: Receiver<(LinkId, TransportPacket)>,
    links: HashMap<LinkId, (Link, (Sender<(LinkId, TransportPacket)>, Receiver<(LinkId, TransportPacket)>))>,
}

impl Copernica {
    pub fn new() -> Self {
        let (inbound_tx, inbound_rx) = unbounded::<(LinkId, TransportPacket)>();
        let (outbound_tx, outbound_rx) = unbounded::<(LinkId, TransportPacket)>();
        let links = HashMap::new();
        Self { inbound_tx, inbound_rx, outbound_tx, outbound_rx, links }
    }

    pub fn create_link(&mut self, link_id: LinkId) -> Result<(Sender<(LinkId, TransportPacket)>, Receiver<(LinkId, TransportPacket)>)> {
        match self.links.get(&link_id) {
            Some(_) => {
                Err(anyhow!("Cannot have two communication channels of the same type with the same id, namely: {:?}", link_id))
            }
            None => {
                let (outbound_tx, outbound_rx) = unbounded::<(LinkId, TransportPacket)>();
                self.links.insert(link_id.clone(), (Link::new(link_id), (outbound_tx, outbound_rx.clone())));
                Ok((self.inbound_tx.clone(), outbound_rx))
            }
        }
    }

    #[allow(unreachable_code)]
    pub fn run(&mut self, db: sled::Db) -> Result<()> {
        //trace!("{:?} IS LISTENING", self.listen_addr);
        //let db = sled::open(self.data_dir.clone())?;
        let inbound_rx = self.inbound_rx.clone();
        let mut links = self.links.clone();
        let outbound_tx = self.outbound_tx.clone();
        let outbound_rx = self.outbound_rx.clone();
        std::thread::spawn(move || {
            loop {
                match inbound_rx.recv() {
                    Ok((from_link_id, tp)) => {
                        if !links.contains_key(&from_link_id) {
                            //trace!("ADDING {:?} to NODE {:?} FACES", link_id, listen_addr.clone());
                            let (outbound_tx, outbound_rx) = unbounded::<(LinkId, TransportPacket)>();
                            links.insert(from_link_id.clone(), (Link::new(from_link_id.clone()), (outbound_tx, outbound_rx)));
                        }
                        Router::handle_packet(&from_link_id, &tp, outbound_tx.clone(), db.clone(), &mut links)?;
                        while !outbound_rx.is_empty() {
                            let (to_link_id, tp) = outbound_rx.recv()?;
                            if let Some((_, (tx, _))) = links.get(&to_link_id) {
                                tx.send((to_link_id, tp))?;
                            }
                        }
                    },
                    Err(error) => return Err(anyhow!("{}", error))
                }
            };
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }
}
