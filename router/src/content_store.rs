use packets::{mk_data, Packet};

#[derive(Clone)]
pub struct ContentStore {
    store: String,
}

impl ContentStore {
    pub fn new() -> Self {
        ContentStore { store: "".to_string() }
    }

    pub fn has_data(&self, i: Packet) -> Option<Packet> {
//        Some(mk_data("stewart".to_string()))
        None
    }
}
