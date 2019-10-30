use crate::{ContentStore};
use packets::{Packet};
use lru::LruCache;
use std::vec::Vec;
use std::sync::Arc;
use std::sync::Mutex;

use {
    log::{info},
};
#[derive(Debug, Clone)]
pub struct InMemory {
    cache: Arc<Mutex<LruCache<Vec<Vec<u16>>, Packet>>>,
}

impl InMemory {
    pub fn new() -> Box<InMemory> {
        Box::new(InMemory {
            cache:  Arc::new(Mutex::new(LruCache::new(2))),
        })
    }

}

impl ContentStore for InMemory {
    fn has_data(&self, sdri: &Vec<Vec<u16>>) -> Option<Packet> {
        match self.cache.lock().unwrap().get(sdri) {
            Some(packet) => {
                Some(packet.clone())
            },
            None => {
                None
            },
        }
    }

    fn put_data(&mut self, data: Packet) {
        match data.clone() {
            Packet::Response { sdri, data: _p_data } => {
                self.cache.lock().unwrap().put(sdri.clone(), data);
            },
            Packet::Request { sdri } => {
                assert_eq!(Packet::Request { sdri: sdri }, data);
            },
        };
    }

    fn box_clone(&self) -> Box<dyn ContentStore> {
        Box::new((*self).clone())
    }
}
