use packets::{Packet};

use lru::LruCache;
use bitvec::prelude::*;

#[derive(Debug, Clone)]
pub struct SparseDistributedRepresentation {
    sdr: BitVec,
    //lru: LruCache<String, Vec<Vec<usize>>>,
}

impl SparseDistributedRepresentation {
    pub fn new() -> Self {
        SparseDistributedRepresentation {
            sdr: bitvec![0; 2048],
         //   lru: LruCache::new(1000),
        }
    }

    pub fn insert(&mut self, packet: Packet) {
        let mut n: String = String::new();
        let mut i: Vec<Vec<u16>> = Vec::new();
        match packet {
            Packet::Interest { name, sdri } => {
                n = name;
                i = sdri;
            },
            Packet::Data { name, sdri } => {
                n = name;
                i = sdri;
            },
        }
        //self.lru.put(name.to_string(), indices.clone());
        for row in i {
            for elem in row {
                self.sdr.set(elem as usize, true);
            }
        }
    }

    pub fn contains(&mut self, packet: Packet) -> u8 {
        let mut n: String = String::new();
        let mut i: Vec<Vec<u16>> = Vec::new();
        let mut sdr_vals: Vec<u32> = Vec::new();
        match packet {
            Packet::Interest { name, sdri } => {
                n = name;
                i = sdri;
            },
            Packet::Data     { name, sdri } => {
                n = name;
                i = sdri;
            },
        }
        for row in i {
            for elem in row {
                sdr_vals.push(self.sdr.get(elem as usize).unwrap() as u32);
            }
        }
        let hits = sdr_vals.iter().try_fold(0u32, |acc, &elem| acc.checked_add(elem));
        let percentage = (hits.unwrap() as f32 / sdr_vals.len() as f32) * 100f32;
        //println!("hits: {:?}, length: {:?}, percentage: {}", hits.unwrap(), vals.len(), percentage);
        percentage as u8
    }

    pub fn delete(&mut self, packet: Packet) {
        let mut n: String = String::new();
        let mut i: Vec<Vec<u16>> = Vec::new();
        let mut sdr_vals: Vec<u32> = Vec::new();
        match packet {
            Packet::Interest { name, sdri } => {
                n = name;
                i = sdri;
            },
            Packet::Data     { name, sdri } => {
                n = name;
                i = sdri;
            },
        }
        for row in i {
            for elem in row {
                self.sdr.set(elem as usize, false);
            }
        }
    }

    //pub fn print(&self) {
    //    println!("{:?}", self.sdr);
    //}
}

impl PartialEq for SparseDistributedRepresentation {
    fn eq(&self, other: &SparseDistributedRepresentation) -> bool {
        self.sdr == other.sdr
    }
}


#[cfg(test)]
mod sdr_tests {
    use super::*;
    use packets::{ mk_interest};

    #[test]
    fn contains_return_100_percent() {
        let interest = mk_interest("interested/in/world/affairs".to_string());
        let mut sdr = SparseDistributedRepresentation::new();
        sdr.insert(interest.clone());
        //println!("{}", sdr.contains(interest.clone()));
        //sdr.print();
        assert_eq!(sdr.contains(interest), 100);
    }
}
