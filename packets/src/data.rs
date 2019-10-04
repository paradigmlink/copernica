#[derive(Serialize, Deserialize, Debug)]
pub struct Data<'a> {
    name: &'a str,
}

impl<'a> Data<'a> {
    pub fn new(name: &str) -> Data {
        Data {
            name : name,
        }
    }
}
