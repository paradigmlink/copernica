use packets::{Packet};
use lru::LruCache;
use std::vec::Vec;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ContentStore{
    cache: Arc<Mutex<LruCache<Vec<Vec<u16>>, Packet>>>,
}

impl ContentStore {
    pub fn new(size: u64) -> ContentStore {
        ContentStore {
            cache:  Arc::new(Mutex::new(LruCache::new(size as usize))),
        }
    }

}

impl ContentStore {
    pub fn has_data(&self, sdri: &Vec<Vec<u16>>) -> Option<Packet> {
        match self.cache.lock().unwrap().get(sdri) {
            Some(packet) => {
                Some(packet.clone())
            },
            None => {
                None
            },
        }
    }

    pub fn put_data(&mut self, data: Packet) {
        match data.clone() {
            Packet::Response { sdri, data: _p_data } => {
                self.cache.lock().unwrap().put(sdri.clone(), data);
            },
            Packet::Request { sdri } => {
                assert_eq!(Packet::Request { sdri: sdri }, data);
            },
        };
    }
}
