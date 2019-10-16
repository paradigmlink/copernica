extern crate bitvec;
extern crate packets;
extern crate rand;
#[macro_use]
extern crate serde_derive;

mod sparse_distributed_representation;
pub mod udp;
pub mod tcp;
pub use crate::{udp::Udp, tcp::Tcp};

use packets::{Packet};
use async_std::io;

pub trait Face {
    fn id(&self) -> u32;
    // router uses these
    fn send_interest_downstream(&mut self, interest: Packet);
    fn receive_upstream_interest(&mut self) -> Option<Packet>;
    fn send_data_upstream(&mut self, data: Packet);
    fn receive_downstream_data(&mut self) -> Option<Packet>;

    fn create_pending_interest(&mut self, interest: Packet);
    fn contains_pending_interest(&mut self, packet: Packet) -> u8;
    fn delete_pending_interest(&mut self, interest: Packet);

    fn create_forwarding_hint(&mut self, interest: Packet);
    fn contains_forwarding_hint(&mut self, interest: Packet) -> u8;
    fn forwarding_hint_decoherence(&mut self) -> u8;
    fn restore_forwarding_hint(&mut self);

    // application uses these
    //fn try_interest(&self) -> Option<Packet>;
    //fn interest(&self) -> Option<Packet>;


    fn box_clone(&self) -> Box::<dyn Face>;
    fn receive(&mut self) -> async_std::io::Result<()> ;
    fn send(&mut self);
    fn print_pi(&self);
    fn print_fh(&self);
}

impl Clone for Box<dyn Face> {
    fn clone(&self) -> Box<dyn Face> {
        self.box_clone()
    }
}
