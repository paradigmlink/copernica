
extern crate bincode;

use thin_waist;

fn main() {
    // The object that we will serialize.
    let target: Option<String>  = Some("hello world".to_string());

    let encoded: Vec<u8> = bincode::serialize(&target).unwrap();
    let decoded: Option<String> = bincode::deserialize(&encoded[..]).unwrap();
    assert_eq!(target, decoded);
    let num = 10;
    println!("Hello, world! {} plus one is {}!", num, thin_waist::add_one(num));
    println!("{:?}", decoded.unwrap());
}
