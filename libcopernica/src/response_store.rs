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
        narrow_waist::{NarrowWaist, mk_response_packet, Bytes},
        sdri::{Sdri},
        constants,
    }
};

#[derive(Debug, Clone)]
pub struct ResponseStore {
    lru: Arc<Mutex<LruCache<Sdri, Response>>>,
}

impl ResponseStore {
    pub fn new(size: u64) -> ResponseStore {
        ResponseStore {
            lru:  Arc::new(Mutex::new(LruCache::new(size as usize))),
        }
    }
    pub fn from_name_and_data(&mut self, name: String, data: Bytes) {
        let chunks = data.chunks(constants::FRAGMENT_SIZE);
        let mut packets: BTreeMap<u64, NarrowWaist> = BTreeMap::new();
        let length = chunks.len() as u64;
        let mut count: u64 = 0;
        for chunk in chunks {
            packets.insert(count.clone(), mk_response_packet(name.clone(), chunk.to_vec(), count, length));
            count += 1;
        }
        let response = Response {
            sdri: Sdri::new(name),
            packets,
            length,
        };
        self.lru.lock().unwrap().put(response.sdri(), response);
    }
    pub fn from_name_and_btreemap(&mut self, name: String, data: BTreeMap<u64, NarrowWaist>) {
        let response = Response::from_name_and_btreemap(name, data);
        self.lru.lock().unwrap().put(response.sdri(), response);
    }
    pub fn get_response(&self, sdri: &Sdri) -> Option<Response> {
        match self.lru.lock().unwrap().get(sdri) {
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
        self.lru.lock().unwrap().put(response.sdri(), response);
    }

    pub fn insert_packet(&mut self, packet: NarrowWaist) {
        match packet.clone() {
            NarrowWaist::Response { sdri, ..} => {
                let mut lru_guard = self.lru.lock().unwrap();
                if let Some(response) = lru_guard.get_mut(&sdri) {
                    response.insert(packet);
                } else {
                    let response = Response::from_response_packet(packet);
                    lru_guard.put(sdri, response);
                }
            },
            NarrowWaist::Request { .. } => panic!("Request should never be inserted into a Response"),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Response {
    sdri: Sdri,
    length: u64,
    packets: BTreeMap<u64, NarrowWaist>,
}

impl Response {
    pub fn insert(&mut self, packet: NarrowWaist) {
        match packet.clone() {
            NarrowWaist::Response { sdri, count, total, .. } => {
                if self.sdri.to_vec() != sdri.to_vec() {
                    panic!("Response.sdri not equal to NarrowWaist::Response{sdri, ..}");
                }
                if self.length == total {
                    self.packets.insert(count, packet);
                } else {
                    panic!("Response.length not equal NarrowWaist::Response{total, ..}");
                }
            },
            NarrowWaist::Request {..} => {
                panic!("Cannot insert a NarrowWaist::Request into a Response");
            },
        }
    }
    pub fn from_name_and_data(name: String, data: Bytes) -> Response {
        let chunks = data.chunks(constants::FRAGMENT_SIZE);
        let mut packets: BTreeMap<u64, NarrowWaist> = BTreeMap::new();
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
    pub fn from_name_and_btreemap(name: String, data: BTreeMap<u64, NarrowWaist>) -> Response {
        Response {
            sdri: Sdri::new(name),
            length: data.len() as u64,
            packets: data.clone(),
        }
    }
    pub fn from_response_packet(packet: NarrowWaist) -> Response {
        match packet.clone() {
            NarrowWaist::Response { sdri, total, ..} => {
                let mut response = Response {
                    sdri,
                    packets: BTreeMap::new(),
                    length: total,
                };
                response.insert(packet);
                return response
            },
            NarrowWaist::Request { .. } => {
                panic!("Cannot create a Response from a NarrowWaist::Request");
             },
        }
    }    pub fn iter(&self) -> std::collections::btree_map::Iter<u64, NarrowWaist> {
        self.packets.iter()
    }
    pub fn payload(&self) -> Bytes {
        self.packets
            .values()
            .cloned()
            .map(|p|
                match p {
                    NarrowWaist::Response { data, ..} => data.clone(),
                    NarrowWaist::Request {..} => panic!("There should be no requests in a Response"),
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
