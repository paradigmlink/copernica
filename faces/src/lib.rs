extern crate bitvec;
extern crate packets;

mod sparse_distributed_representation;
pub mod tcp;
pub use crate::{tcp::Tcp};

use packets::{Packet};

pub trait Face {
    fn id(&self) -> u8;
    fn send_interest_downstream(&mut self, interest: Packet);
    fn receive_upstream_interest(&mut self) -> Option<Packet>;
    fn send_data_upstream(&mut self, data: Packet);
    fn receive_downstream_data(&mut self) -> Option<Packet>;
    fn create_pending_interest(&mut self, interest: Packet);
    fn contains_forwarding_hint(&mut self, interest: Packet) -> u8;
    fn create_forwarding_hint(&mut self, interest: Packet);
    fn contains_pending_interest(&mut self, packet: Packet) -> u8;
    fn delete_pending_interest(&mut self, interest: Packet);

    fn box_clone(&self) -> Box::<Face>;
}

impl Clone for Box<dyn Face> {
    fn clone(&self) -> Box<dyn Face> {
        self.box_clone()
    }
}
