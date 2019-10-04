#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    name: String,
}

impl Data {
    pub fn new(name: String) -> Data {
        Data {
            name : name,
        }
    }
}

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

