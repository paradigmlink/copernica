
#[derive(Clone)]
pub struct ContentStore {
    store: String,
}

impl ContentStore {
    pub fn new() -> Self {
        ContentStore { store: "".to_string() }
    }
}
