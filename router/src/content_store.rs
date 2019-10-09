use packets::{Interest, Data};

#[derive(Clone)]
pub struct ContentStore {
    store: String,
}

impl ContentStore {
    pub fn new() -> Self {
        ContentStore { store: "".to_string() }
    }

    pub fn has_data(&self, i: Interest) -> Option<Data> {
        Some(Data::new("stewart".to_string()))
    }
}
