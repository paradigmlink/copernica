use {
    copernica::{
        InterLinkPacket, Link
    },
    crossbeam_channel::{Sender, Receiver},
    anyhow::{Result},
};
pub mod udp;
pub mod mpsc_channel;
pub use {
    udp::{UdpIp},
    mpsc_channel::{MpscChannel},
};

pub trait Transport<'a> {
    //fn bind(&mut self) -> Receiver<TransportPacket>;
    //fn bind_with(&mut self, copernica_to_transport_rx: Receiver<TransportPacket>);
    fn run(&self) -> Result<()>;
    fn new(link: Link, router_in_and_out: ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> ) ) -> Result<Self> where Self: Sized;
}
