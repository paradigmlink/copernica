use {
    crate::{Transport},
    copernica::{
        TransportPacket, ReplyTo, LinkId
    },
    anyhow::{anyhow, Result},
    crossbeam_channel::{Sender, Receiver, unbounded},
};

pub struct MpscChannel {
    link_id: LinkId,
    router_inbound_tx: Sender<(LinkId, TransportPacket)>,
    router_outbound_rx: Receiver<(LinkId, TransportPacket)>,
    network_outbound_tx: Sender<(LinkId, TransportPacket)>,
    network_inbound_rx: Receiver<(LinkId, TransportPacket)>,
    links: Vec<Sender<(LinkId, TransportPacket)>>,
}

impl MpscChannel {
    pub fn peer_info(&mut self) -> Sender<(LinkId, TransportPacket)> {
        self.network_outbound_tx.clone()
    }

    pub fn peer_with(&mut self, network_outbound_tx: Sender<(LinkId, TransportPacket)>) {
        self.links.push(network_outbound_tx);
    }
    pub fn unbounded(&self) -> ( Sender<(LinkId, TransportPacket)> , Receiver<(LinkId, TransportPacket)> ) {
        (self.network_outbound_tx.clone(), self.network_inbound_rx.clone())
    }
}

impl<'a> Transport<'a> for MpscChannel {
    fn new(link_id: LinkId
        , (router_inbound_tx, router_outbound_rx): ( Sender<(LinkId, TransportPacket)> , Receiver<(LinkId, TransportPacket)> )
        ) -> Result<MpscChannel>
    {
        match link_id.reply_to() {
            ReplyTo::Mpsc => {
                let links = vec![];
                let (network_outbound_tx, network_inbound_rx) = unbounded::<(LinkId, TransportPacket)>();
                return Ok(MpscChannel { link_id, network_outbound_tx, network_inbound_rx, router_inbound_tx, router_outbound_rx, links })
            }
            _ => return Err(anyhow!("MpscChannel Transport expects a LinkId of type ReplyTo::Mpsc")),
        }
    }

    #[allow(unreachable_code)]
    fn run(&self) -> Result<()> {
        let link_id = self.link_id.clone();
        let network_inbound_rx = self.network_inbound_rx.clone();
        let router_inbound_tx = self.router_inbound_tx.clone();
        std::thread::spawn(move || {
            match link_id.reply_to() {
                ReplyTo::Mpsc => {
                    loop {
                        match network_inbound_rx.recv(){
                            Ok(msg) => {
                                let _r = router_inbound_tx.send(msg)?;
                            },
                            Err(error) => return Err(anyhow!("{}", error)),
                        };
                    }
                },
                _ => {},
            }
            Ok::<(), anyhow::Error>(())
        });
        let link_id = self.link_id.clone();
        let router_outbound_rx = self.router_outbound_rx.clone();
        let links = self.links.clone();
        std::thread::spawn(move || {
            loop {
                match router_outbound_rx.recv(){
                    Ok((_, transport_packet)) => {
                        for network_outbound_tx in &links {
                            let transport_packet = TransportPacket::new(link_id.clone().reply_to(), transport_packet.payload());
                            network_outbound_tx.send((link_id.clone(), transport_packet))?;
                        }
                    },
                    Err(error) => return Err(anyhow!("{}", error)),
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }
}

