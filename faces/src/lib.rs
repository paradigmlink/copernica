extern crate bitvec;
extern crate packets;

pub mod mock;
pub use crate::{mock::Mock};

use packets::{Interest, Data};

pub trait Face {
    fn new() -> Self where Self: Sized;
    fn interest_in(&self, i: Interest);
    fn interest_poll(&self) -> Option<Interest>;
    fn data_in(&self, d: Data);
    fn data_poll(&self) -> Option<Data>;
}
