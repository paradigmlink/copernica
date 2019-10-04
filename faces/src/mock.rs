use crate::Face;

use bitvec::prelude::*;
use packets::{Interest, Data};
use crossbeam_channel::{unbounded, Sender, Receiver};

pub struct Mock<'a> {
    sdr: BitVec,
    i_in: Sender<Interest<'a>>,
    i_out: Receiver<Interest<'a>>,
    d_in: Sender<Data<'a>>,
    d_out: Receiver<Data<'a>>,
}

impl<'a> Mock<'a> {
}

impl<'a> Face for Mock<'a> {
    fn new() -> Mock<'a> {
        let (i_in, i_out) = unbounded();
        let (d_in, d_out) = unbounded();
        Mock { sdr: bitvec![0; 2048], i_in: i_in, i_out: i_out, d_in: d_in, d_out: d_out }
    }
    fn interest_in<'b>(&self, i: Interest<'b>) {
        self.i_in.send(i).unwrap();
    }
    fn interest_out(&self) -> Interest {
        self.i_out.recv().unwrap()
    }
    fn data_in(&self, d: Data) {
    }
    fn data_out(&self) -> Data {
        Data::new("blah")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_and_receive_interest_and_data() {
        let mock_face: Mock = Face::new();
        let interest = Interest::new("blah");
        mock_face.interest_in(interest);
//        println!("interest out: {:?}" mock_face.interest_out());
        assert_eq!(interest, mock_face.interest_out());
    }
}
