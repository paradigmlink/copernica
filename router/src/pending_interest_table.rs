
#[derive(Clone)]
pub struct PendingInterestTable{
    store: String,
}

impl PendingInterestTable {
    pub fn new() -> Self {
        PendingInterestTable { store: "".to_string() }
    }
}
