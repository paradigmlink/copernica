extern crate bitvec;
extern crate packets;

mod sparse_distributed_representation;
pub mod mock;
pub use crate::{mock::Mock};

/*
use packets::{Packet};

pub trait Face {
    fn new() -> Self where Self: Sized;
    fn id(&self) -> u8;
    fn send_interest_downstream(&mut self, i: Interest);
    fn receive_upstream_interest(&mut self) -> Option<Interest>;
    fn send_data_upstream(&mut self, d: Data);
    fn receive_downstream_data(&mut self) -> Option<Data>;
    fn create_pending_interest(&mut self, interest: Interest);
    fn create_breadcrumb_trail(&mut self, interest: Interest);
    fn contains_forwarding_hint(&mut self, interest: Interest) -> u8;
    fn contains_pending_interest(&mut self, interest: Interest) -> u8;
    fn delete_pending_interest(&mut self, interest: Interest);
}
*/
