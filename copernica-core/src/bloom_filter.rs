use {
    crate::{hbfi::{HBFI, BFI},},
    std::{ collections::HashMap, },
};

#[derive(Clone)]
pub struct BloomFilter {
    rc: HashMap<Vec<BFI>, u64>,
}

impl BloomFilter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        BloomFilter {
            rc: HashMap::new(),
        }
    }

    pub fn insert(&mut self, packet: &HBFI) {
        *self.rc.entry(packet.to_vec()).or_insert(0) += 1;
    }

    pub fn contains(&self, packet: &HBFI) -> bool {
        if let Some(contains) = self.rc.get(&packet.to_vec()) {
            if contains > &0 {
                return true
            } else {
                return false
            }
        } else {
            return false
        }
    }

    pub fn delete(&mut self, packet: &HBFI) {
        *self.rc.entry(packet.to_vec()).or_insert(0) -= 1;
    }
}

