use {
    copernica::{
        TransportPacket, LinkId
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
    fn new(link_id: LinkId, router_in_and_out: ( Sender<(LinkId, TransportPacket)> , Receiver<(LinkId, TransportPacket)> ) ) -> Result<Self> where Self: Sized;
    fn run(&self) -> Result<()>;
}
