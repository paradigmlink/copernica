use base64::{encode, decode};
use sha3::{Digest, Sha3_512};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Interest {
    name: String,
}

impl Interest {
//    pub fn new(s: String) -> Self {
//    }
}

fn hash_name(s: &str) -> String {
    let mut hasher = Sha3_512::new();
    hasher.input(s.as_bytes());
    format!("{:x}", hasher.result())
}

fn index_of_lowest_occuring_char_in_hash<'a>( hash: &'a String) -> Vec<(usize, &'a str)> {
    let chars = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f"];
    let mut curr_min: Vec<(usize, &str)> = hash.match_indices("0").collect();
    for c in chars.iter() {
        let index: Vec<(usize, &str)> = hash.match_indices(c).collect();
        if index.len() < curr_min.len() {
            curr_min = index;
        }
    }
    curr_min
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_index_of_char_in_hash() {
        let h1 = hash_name("domain/app/data");
        let h2 = hash_name("stewart");
        let h3 = hash_name("calculus");
        let h4 = hash_name("topology");
        let h5 = hash_name("mozart");
        let h6 = hash_name("Johann Sebastian Bach");
        let h7 = hash_name("abracadabra");
        let h8 = hash_name("abc");
        assert_eq!(vec![(11, "7"), (54 , "7"), (76 , "7"), (98 , "7")],              index_of_lowest_occuring_char_in_hash(&h1));
        assert_eq!(vec![(70, "6"), (80 , "6"), (96 , "6"), (103, "6")],              index_of_lowest_occuring_char_in_hash(&h2));
        assert_eq!(vec![(22, "3"), (24 , "3"), (26 , "3"), (42 , "3") , (110, "3")], index_of_lowest_occuring_char_in_hash(&h3));
        assert_eq!(vec![(45, "4"), (99 , "4"), (106, "4")],                          index_of_lowest_occuring_char_in_hash(&h4));
        assert_eq!(vec![(33, "e"), (117, "e"), (125, "e")],                          index_of_lowest_occuring_char_in_hash(&h5));
        assert_eq!(vec![(64, "d"), (97 , "d"), (110, "d"), (111, "d")],              index_of_lowest_occuring_char_in_hash(&h6));
        assert_eq!(vec![(9 , "1"), (59 , "1"), (72 , "1"), (109, "1")],              index_of_lowest_occuring_char_in_hash(&h7));
        assert_eq!(vec![(9 , "a"), (15 , "a"), (75 , "a"), (81 , "a") , (110, "a")], index_of_lowest_occuring_char_in_hash(&h8));

    }

    #[test]
    fn sha3_512_a_name() {
        assert_eq!(hash_name("domain/app/data"),
        "0d59b939e0a7f3c152a1186561b13e8f989e8f15513ffa198b65007a6aad16c92e2513fd44a17a3c252e3d48ccc05a9a89781b44acafb5a552eed5314c229bf2");
        assert_eq!(hash_name("stewart"),
        "0a1574102ea3038fa5930a4a8443debe3e04c020cddf5797af98bdc8829d1d8ae3bbaf641ca4794361d13db8deab25d76fdece76a9f829d050807f851d411f85");
        assert_eq!(hash_name("calculus"),
        "5f90e72ee074acd50c898031323d669d8cbba7a1fe3d17b974de0f20c0119fb97ef2a189c42bfea2682a61e49d7cc191b1b491464b50c5321d4fb2c06bc7a5d0");
        assert_eq!(hash_name("topology"),
        "a7118fa98c386af512ff0957603c2fdb6a5a868dc1ed84c37c1aae1bc78f2a8adefddab191859cc732be27a513be25cc15a4c8306e4c59c11efd0219229b9c2b");
        assert_eq!(hash_name("mozart"),
        "4f2b32c88ff5fc42b802aa38b5440f87ded97d183113313af39524451995bd84851afd5bf3cc37729c7c4711601b721661f24dd17870faf366f23e93a17ddeba");
        assert_eq!(hash_name("Johann Sebastian Bach"),
        "a9a60e87c456f5244fc99ec2c4002be9ef24afb9aca1f0cc253f55319e8f364ed01095b2167903b5e6c53ef4427915f72dc429146be567dd81897ce3cf6585ac");
        assert_eq!(hash_name("abracadabra"),
        "2707e4cb41c052cb55846c8f083dcfc0682c920cf8c835df3e92a69b0a41c69d06970e681e0c6f733265644a95bd6f0829880c56572f31255eca450322b960c9");
        assert_eq!(hash_name("abc"),
        "b751850b1a57168a5693cd924b6b096e08f621827444f70d884f5d0240d2712e10e116e9192af3c91a7ec57647e3934057340b4cf408d5a56592f8274eec53f0");
    }

    #[test]
    fn base64_encode() {
        let b64 = encode(b"domain/app/data");
        let txt = decode(&b64);
        assert_eq!(b"domain/app/data", &txt.unwrap()[..]);
    }

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
        assert_eq!(encoded.len(), 9);
    }
}
