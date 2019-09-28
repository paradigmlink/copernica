#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Interest {
    pub name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Data {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_encode_decode() {
        let interest = Interest{ name: "interest".to_string() };
        let encoded = bincode::serialize(&interest).unwrap();
        let decoded = bincode::deserialize(&encoded).unwrap();
        assert_eq!(interest, decoded);
    }

    #[test]
    fn encode_length() {
        let target = "a".to_string();
        let encoded: Vec<u8> = bincode::serialize(&target).unwrap();
        println!("{:?}",encoded);
        assert_eq!(encoded.len(), 9);
    }
}
