use {
    crate::{
        packets::{Packet, Sdri, Response},
    },
    lru::LruCache,
    std::{
        sync::{
            Arc,
            Mutex,
        },
        collections::{
            BTreeMap,
        },
    },
};

#[derive(Debug, Clone)]
pub struct ContentStore{
    cache: Arc<Mutex<LruCache<Sdri, Response>>>,
}

impl ContentStore {
    pub fn new(size: u64) -> ContentStore {
        ContentStore {
            cache:  Arc::new(Mutex::new(LruCache::new(size as usize))),
        }
    }

}

impl ContentStore {
    pub fn get_response(&self, sdri: &Sdri) -> Option<Response> {
        match self.cache.lock().unwrap().get(sdri) {
            Some(response) => {
                if response.complete() {
                    return Some(response.clone())
                } else {
                    return None
                }
            },
            None => {
                None
            },
        }
    }

    pub fn insert_response(&mut self, response: Response) {
        self.cache.lock().unwrap().put(response.sdri(), response);
    }

    pub fn insert_packet(&mut self, packet: Packet) {
        match packet.clone() {
            Packet::Response { sdri, ..} => {
                let mut cache_guard = self.cache.lock().unwrap();
                if let Some(response) = cache_guard.get_mut(&sdri) {
                    response.insert(packet);
                } else {
                    let response = Response::from_response_packet(packet);
                    cache_guard.put(sdri, response);
                }
            },
            Packet::Request { .. } => panic!("Request should never be inserted into a Response"),
        }
    }
}
