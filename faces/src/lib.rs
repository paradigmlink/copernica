extern crate bitvec;
extern crate packets;

pub mod mock;

use packets::{Interest, Data};

pub trait Face {
    fn new() -> Self;
    fn interest_in(&self, i: Interest);
    fn interest_out(&self) -> Interest;
    fn data_in(&self, d: Data);
    fn data_out(&self) -> Data;
}
