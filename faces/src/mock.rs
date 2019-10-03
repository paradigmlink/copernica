use crate::Face;

use bitvec::prelude::*;
use packets::{Interest, Data};

#[derive(PartialEq, Debug)]
pub struct Mock {
    sdr: BitVec,
}

impl Mock {
    pub fn new(bv: BitVec) -> Mock {
        Mock { sdr : bv }
    }
}

impl Face for Mock {
    fn new(&mut self) -> Mock {
        Mock::new(bitvec![0; 2048])
    }
    fn interest_in(&self, i: Interest) {
    }
    fn interest_out(&self) -> Interest {
        Interest::new("blah")
    }
    fn data_in(&self, d: Data) {
    }
    fn data_out(&self) -> Data {
        Data::new("blah")
    }
}

