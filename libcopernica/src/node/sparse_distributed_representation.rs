// https://numenta.com/assets/pdf/biological-and-machine-intelligence/BaMI-SDR.pdf
use {
    rand::Rng,
    bitvec::prelude::*,
    crate::sdri::{Sdri},
};

#[derive(Debug, Clone)]
pub struct SparseDistributedRepresentation {
    sdr: BitVec,
}

impl SparseDistributedRepresentation {
    pub fn new() -> Self {
        SparseDistributedRepresentation {
            sdr: bitvec![0; 2048],
        }
    }

    pub fn insert(&mut self, packet: &Sdri) {
        for row in packet.to_vec() {
            for elem in row {
                self.sdr.set(elem as usize, true);
            }
        }
    }

    pub fn contains(&self, packet: &Sdri) -> u8 {
        let mut sdr_vals: Vec<u32> = Vec::new();
        for row in packet.to_vec() {
            for elem in row {
                sdr_vals.push(*self.sdr.get(elem as usize).unwrap() as u32);
            }
        }
        let hits = sdr_vals.iter().try_fold(0u32, |acc, &elem| acc.checked_add(elem));
        let percentage = (hits.unwrap() as f32 / sdr_vals.len() as f32) * 100f32;
        //println!("hits: {:?}, length: {:?}, percentage: {}", hits.unwrap(), vals.len(), percentage);
        percentage as u8
    }

    pub fn delete(&mut self, packet: &Sdri) {
        for row in packet.to_vec() {
            for elem in row {
                self.sdr.set(elem as usize, false);
            }
        }
    }

    pub fn partially_forget(&mut self) {
        let mut rng = rand::thread_rng();
        for _ in 0 .. 2048 {
            self.sdr.set(rng.gen_range(0, 2048), false);
        }
    }

    pub fn decoherence(&self) -> u8 {
        let hits = self.sdr.iter().try_fold(0u32, |acc, elem| acc.checked_add(*elem as u32));
        let percentage = (hits.unwrap() as f32 / self.sdr.len() as f32) * 100f32;
        //println!("hits: {:?}, length: {:?}, percentage: {}", hits.unwrap(), vals.len(), percentage);
        percentage as u8
    }

}

impl PartialEq for SparseDistributedRepresentation {
    fn eq(&self, other: &SparseDistributedRepresentation) -> bool {
        self.sdr == other.sdr
    }
}

