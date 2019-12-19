use {
    std::{
        collections::{
            BTreeMap,
        },
        sync::{
            Arc,
            Mutex,
        },
    },
    lru::LruCache,
    crate::{
        packets::{Packet, mk_response_packet, Bytes},
        sdri::{Sdri},
        constants,
    }
};

#[derive(Debug, Clone)]
pub struct ResponseStore {
    cache: Arc<Mutex<LruCache<Sdri, Response>>>,
}

impl ResponseStore {
    pub fn new(size: u64) -> ResponseStore {
        ResponseStore {
            cache:  Arc::new(Mutex::new(LruCache::new(size as usize))),
        }
    }
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

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Response {
    sdri: Sdri,
    length: u64,
    packets: BTreeMap<u64, Packet>,
}

impl Response {
    pub fn from_name_and_data(name: String, data: Bytes) -> Response {
        let chunks = data.chunks(constants::FRAGMENT_SIZE);
        let mut packets: BTreeMap<u64, Packet> = BTreeMap::new();
        let length = chunks.len() as u64;
        let mut count: u64 = 0;
        for chunk in chunks {
            packets.insert(count.clone(), mk_response_packet(name.clone(), chunk.to_vec(), count, length));
            count += 1;
        }
        Response {
            sdri: Sdri::new(name),
            packets,
            length,
        }
    }
    pub fn from_name_and_btreemap(name: String, data: BTreeMap<u64, Packet>) -> Response {
        Response {
            sdri: Sdri::new(name),
            length: data.len() as u64,
            packets: data.clone(),
        }
    }
    pub fn from_response_packet(packet: Packet) -> Response {
        match packet.clone() {
            Packet::Response { sdri, denominator, ..} => {
                let mut response = Response {
                    sdri,
                    packets: BTreeMap::new(),
                    length: denominator,
                };
                response.insert(packet);
                return response
            },
            Packet::Request { .. } => {
                panic!("Cannot create a Response from a Packet::Request");
             },
        }
    }
    pub fn insert(&mut self, packet: Packet) {
        match packet.clone() {
            Packet::Response { sdri, numerator, denominator, .. } => {
                if self.sdri.to_vec() != sdri.to_vec() {
                    panic!("Response.sdri not equal to Packet::Response{sdri, ..}");
                }
                if self.length == denominator {
                    self.packets.insert(numerator, packet);
                } else {
                    panic!("Response.length not equal Packet::Response{denominator, ..}");
                }
            },
            Packet::Request {..} => {
                panic!("Cannot insert a Packet::Request into a Response");
            },
        }
    }
    pub fn iter(&self) -> std::collections::btree_map::Iter<u64, Packet> {
        self.packets.iter()
    }
    pub fn payload(&self) -> Bytes {
        self.packets
            .values()
            .cloned()
            .map(|p|
                match p {
                    Packet::Response { data, ..} => data.clone(),
                    Packet::Request {..} => panic!("There should be no requests in a Response"),
                })
            .flatten()
            .collect()
    }
    pub fn sdri(&self) -> Sdri {
        self.sdri.clone()
    }
    pub fn complete(&self) -> bool {
        self.packets.len() as u64 == self.length
    }
    pub fn expected_length(&self) -> u64 {
        self.length
    }
    pub fn actual_length(&self) -> u64 {
        self.packets.len() as u64
    }
}

pub fn mk_response(name: String, data: Bytes) -> Response {
    Response::from_name_and_data(name, data)
}
