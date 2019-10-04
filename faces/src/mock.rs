use crate::Face;

use bitvec::prelude::*;
use packets::{Interest, Data};
use crossbeam_channel::{unbounded, Sender, Receiver};


#[derive(Debug, Clone)]
pub struct Mock {
    sdr: BitVec,
    i_in: Sender<Interest>,
    i_out: Receiver<Interest>,
    d_in: Sender<Data>,
    d_out: Receiver<Data>,
}

impl Face for Mock {
    fn new() -> Mock {
        let (i_in, i_out) = unbounded();
        let (d_in, d_out) = unbounded();
        Mock { sdr: bitvec![0; 2048], i_in: i_in, i_out: i_out, d_in: d_in, d_out: d_out }
    }
    fn interest_in(&self, i: Interest) {
        println!("interest_in");
        self.i_in.send(i).unwrap();
    }
    fn interest_out(&self) -> Interest {
        println!("interest_out");
        self.i_out.recv().unwrap()
    }
    fn data_in(&self, d: Data) {
        self.d_in.send(d).unwrap();
    }
    fn data_out(&self) -> Data {
        self.d_out.recv().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_face_send_and_receive_interest() {
        let mock_face: Mock = Face::new();
        let interest = Interest::new("blah".to_string());
        mock_face.interest_in(interest.clone());
        assert_eq!(interest, mock_face.interest_out());
    }

    #[test]
    fn mock_face_send_and_receive_data() {
        let mock_face: Mock = Face::new();
        let data = Data::new("blah".to_string());
        mock_face.data_in(data.clone());
        assert_eq!(data, mock_face.data_out());
    }
}
