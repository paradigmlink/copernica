use {
    std::{
        fmt,
        collections::{
            HashMap,
            BTreeMap,
        },
    },
    sha3::{
        Digest,
        Sha3_512,
    },
    chain_addr,
    chain_crypto::bech32::Bech32,
};

const HEX : [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f"];

const SAFE_MTU: usize = 1400;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Sdri {
    id: Vec<u16>,
    name: Option<Vec<u16>>,
    seq: Option<usize>,
}

impl Sdri {
    pub fn new(s: String) -> Sdri {
        let sections = s.splitn(3,"::");
        let sections: Vec<&str> = sections.collect();
        let sdri: Sdri = match sections.len() {
            3 => {
                let name = format!("{}{}", sections[0], sections[1]);
                let seq = sections[2].parse::<usize>().unwrap();
                Sdri {
                    id: name_sparsity(sections[0]),
                    name: Some(name_sparsity(name.as_str())),
                    seq: Some(seq),
                }
            }
            2 => {
                let name = format!("{}{}", sections[0], sections[1]);
                Sdri {
                    id: name_sparsity(sections[0]),
                    name: Some(name_sparsity(name.as_str())),
                    seq: None,
                }
            },
            1 => {
                Sdri {
                    id: name_sparsity(sections[0]),
                    name: None,
                    seq: None,
                }
            },
            _ => unreachable!()
        };
        sdri
    }

    pub fn to_vec(&self) -> Vec<Vec<u16>> {
        let mut out: Vec<Vec<u16>> = vec![];
        out.push(self.id.clone());
        if let Some(name) = self.name.clone() {
            out.push(name);
        }
        //if let Some(seq) = self.seq.clone() {
        //    out.push(seq);
        //}
        out
    }
}

impl fmt::Debug for Sdri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Sdri { id, name: Some(name), seq: Some(seq) } => write!(f, "{:?}::{:?}::{:?}", id, name, seq),
            Sdri { id, name: Some(name), seq: None } => write!(f, "{:?}::{:?}", id, name),
            Sdri { id, name: None, seq: Some(seq) } => write!(f, "Cannot have a Some(seq) with a None Name; ID::NONE::Seq"),
            Sdri { id, name: None, seq: None } => write!(f, "{:?}", id),
        }
    }
}

pub type Bytes = Vec<u8>;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Packet {
    Request     { sdri: Sdri },
    Response    { sdri: Sdri, data: Bytes, numerator: u64, denominator: u64 },
}

pub fn mk_request_packet(name: String) -> Packet {
    Packet::Request {
        sdri: Sdri::new(name)
    }
}

pub fn mk_response_packet(name: String, data: Bytes, numerator: u64, denominator: u64) -> Packet {
    Packet::Response {
        sdri: Sdri::new(name),
        data,
        numerator,
        denominator,
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Response {
    sdri: Sdri,
    length: u64,
    packets: BTreeMap<u64, Packet>,
}

impl Response {
    pub fn from_name_and_data(name: String, data: Bytes) -> Response {
        let chunks = data.chunks(SAFE_MTU);
        let mut packets: BTreeMap<u64, Packet> = BTreeMap::new();
        let length = (chunks.len() as u64);
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
    pub fn from_name_and_btreemap(name: String, data: BTreeMap<u64, Packet>) -> Response {
        Response {
            sdri: Sdri::new(name),
            length: data.len() as u64,
            packets: data.clone(),
        }
    }
    pub fn from_response_packet(packet: Packet) -> Response {
        match packet.clone() {
            Packet::Response { sdri, denominator, ..} => {
                let mut response = Response {
                    sdri,
                    packets: BTreeMap::new(),
                    length: denominator,
                };
                response.insert(packet);
                return response
            },
            Packet::Request { .. } => {
                panic!("Cannot create a Response from a Packet::Request");
             },
        }
    }
    pub fn insert(&mut self, packet: Packet) {
        match packet.clone() {
            Packet::Response { sdri, numerator, denominator, .. } => {
                if self.sdri.to_vec() != sdri.to_vec() {
                    panic!("Response.sdri not equal to Packet::Response{sdri, ..}");
                }
                if self.length == denominator {
                    self.packets.insert(numerator, packet);
                } else {
                    panic!("Response.length not equal Packet::Response{denominator, ..}");
                }
            },
            Packet::Request {..} => {
                panic!("Cannot insert a Packet::Request into a Response");
            },
        }
    }
    pub fn iter(&self) -> std::collections::btree_map::Iter<u64, Packet> {
        self.packets.iter()
    }
    pub fn payload(&self) -> Bytes {
        self.packets
            .values()
            .cloned()
            .map(|p|
                match p {
                    Packet::Response { data, ..} => data.clone(),
                    Packet::Request {..} => panic!("There should be no requests in a Response"),
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

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Packet::Request{sdri} => write!(f, "REQ{:?}", sdri),
            Packet::Response{sdri, numerator, denominator, ..} =>
                write!(f, "RES{:?}_{}/{}", sdri, numerator, denominator-1)
        }
    }
}

fn hash_name(s: &str) -> String {
    let mut hasher = Sha3_512::new();
    hasher.input(s.as_bytes());
    format!("{:x}", hasher.result())
}

fn index_of_lowest_occuring_char_in_hash<'a>( hash: &'a str) -> Vec<(u16, &'a str)> {
    let mut old: Vec<(usize, &str)> = vec![(0,""); 15]; // initialize with a count of 15 occurances for the number 0 that is obviously bigger than most
    for c in HEX[0..].iter() {
        let new: Vec<(usize, &str)> = hash.match_indices(c).collect();
        if new.len() > 1 && new.len() < old.len()  {
            // @Sparsity: running the words list yields an entire byte of 1s! meaning the way this is being done
            // could be dodgy. Look at trying to making sparsity exactly 3 bitvec index elements, maybe feed an RNG
            // with the extracted indices and generate a reproducible index per name.
            //println!("new len: {}, old len: {}", new.len(), old.len());
            old = new;
        }
    }
    old.iter().map(|(x, y)| (*x as u16, *y)).collect()
}

fn gen_2048_sparsity(u: Vec<(u16, &str)>) -> Vec<u16> {
    // x = position of 0 and 128 (length of SHA3_512 hash
    // 16 is used to calculate the bit position
    // y is the character of the position chosen in the hash which gets converted into decimal
    u.iter().map(|(position, character)| (position * 16 + (u16::from_str_radix(character, 16).unwrap()))).collect::<Vec<u16>>()
}

fn name_sparsity(s: &str) -> Vec<(u16)> {
    gen_2048_sparsity(index_of_lowest_occuring_char_in_hash(&hash_name(s)))
}

/*pub fn generate_sdr_index(s: String) -> Sdri {
    let names = s.split('-');
    let names: Vec<&str> = names.collect();
    let mut fh: Sdri = Vec::new();
    for name in names {
        fh.push(name_sparsity(name));
    }
    fh
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{BufRead, BufReader};
    use bitvec::prelude::*;

    #[test]
    fn names() {
        let s: String = "blue::cheese::42".to_string();
        let s: String = "ceo1q0te4aj3u2llwl4mxuxnjm9skj897hncanvgcnz0gf3x57ap6h7gk4dw8nv::my-excel-file.xls".to_string();

        let actual = Sdri::new(s);
        let actual = actual.to_vec();
        let expected = vec![vec![290, 642, 1490], vec![17, 481, 593]];
        println!("{:?}", actual);
        assert_eq!(actual, expected);
    }
/*
    #[test]
    fn load_test_sdr() {
        let mut exe_path = std::env::current_exe().unwrap().to_path_buf();
        exe_path.pop();
        exe_path.pop();
        exe_path.pop();
        exe_path.pop();
        exe_path.push("packets/tests/words.tar.gz");

        let tar_gz = fs::File::open(&exe_path).unwrap();
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack("tests").unwrap();

        let file = fs::File::open(exe_path.with_file_name("words.txt")).unwrap();
        let reader = BufReader::new(file);
    	let mut elts = [0u8; 256]; //2048 bit vector
    	let bs = BitSlice::<BigEndian, _>::from_slice_mut(&mut elts[..]);
        fn is_all_true(arr: &[bool]) -> bool {
            for i in arr {
                if i == &false { return false }
            }
            return true
        }
        let mut break_on_line = 0;
        for (index, line) in reader.lines().enumerate() {
            let line = line.unwrap(); // Ignore errors.
        	let mut first_hit: Vec<bool> = Vec::new();
        	
        	let sdrs = name_sparsity(line.as_str());
        	//print!("index: {}, word: {}, ",index, line);
        	for sdr in &sdrs {
        	    first_hit.push(bs.get(*sdr as usize).unwrap());
        	    //print!("{:?} ", bs.get(*sdr).unwrap());
        	    bs.set(*sdr as usize, true);
        	}
        	//println!("\n");
        	if is_all_true(first_hit.as_slice()) {
        	    //println!("BitVector:\n {}", &bs);
        	    break_on_line = index;
            	break
            }
        }
        fs::remove_file(exe_path.with_file_name("words.txt")).unwrap();
        assert_eq!(break_on_line, 234); // this number should only get higher
        // but later on smaller routers on the edge will want to have smaller bitvectors
        // which means it can hold less information.


    }
*/

    #[test]
    fn test_creation_of_sparsity_in_a_2048_bit_vector() {
        let h1 = hash_name("AAAL");
        let h2 = hash_name("AB");
        let h3 = hash_name("2");
        let h4 = hash_name("ceo1q0te4aj3u2llwl4mxuxnjm9skj897hncanvgcnz0gf3x57ap6h7gk4dw8nv");
        //test the max level i.e. ffff in the last position, make sure it fits in a 2048 bit vector!
        let h5 = "564e1971233e198c2614121214e652742355e6161e181a881c97511869aac1c29c1944175c374a716769989aa7a4216198ee1215311782785111e2854158ffff".to_string();
        let h6 = "564119712331198c26141212141652742355161611181a881c97511869aac1c29c1944175c374a716769989aa7a421619811121531178278511112854158eeee".to_string();
        assert_eq!(vec![289, 305, 1137, 1377, 1633], gen_2048_sparsity(index_of_lowest_occuring_char_in_hash(&h1)));
        assert_eq!(vec![355, 515, 1795, 1875], gen_2048_sparsity(index_of_lowest_occuring_char_in_hash(&h2)));
        assert_eq!(vec![192, 848, 1808, 1936], gen_2048_sparsity(index_of_lowest_occuring_char_in_hash(&h3)));
        assert_eq!(vec![290, 642, 1490], gen_2048_sparsity(index_of_lowest_occuring_char_in_hash(&h4)));
        assert_eq!(vec![1999, 2015, 2031, 2047], gen_2048_sparsity(index_of_lowest_occuring_char_in_hash(&h5)));
        assert_eq!(vec![1998, 2014, 2030, 2046], gen_2048_sparsity(index_of_lowest_occuring_char_in_hash(&h6)));
    }

    #[test]
    fn find_index_of_char_in_hash() {
        let h1 = hash_name("AAAL");
        let h2 = hash_name("AB");
        let h3 = hash_name("2");
        let h4 = hash_name("AAA");
        let h5 = hash_name("ceo1q0te4aj3u2llwl4mxuxnjm9skj897hncanvgcnz0gf3x57ap6h7gk4dw8nv");
        let h6 = "564e1971233e198c2614121214e652742355e6161e181a881c97511869aac1c29c1944175c374a716769989aa7a4216198ee1215311782785111e2854158ffff".to_string();
        let h7 = "564119712331198c26141212141652742355161611181a881c97511869aac1c29c1944175c374a716769989aa7a421619811121531178278511112854158eeee".to_string();
        assert_eq!(vec![(18, "1"), (19, "1"), (71, "1"), (86, "1"), (102, "1")], index_of_lowest_occuring_char_in_hash(&h1));
        assert_eq!(vec![(22, "3"), (32, "3"), (112, "3"), (117, "3")], index_of_lowest_occuring_char_in_hash(&h2));
        assert_eq!(vec![(12, "0"), (53, "0"), (113, "0"), (121, "0")], index_of_lowest_occuring_char_in_hash(&h3));
        assert_eq!(vec![(3, "c"), (16, "c"), (65, "c")], index_of_lowest_occuring_char_in_hash(&h4));
        assert_eq!(vec![(18, "2"), (40, "2"), (93, "2")], index_of_lowest_occuring_char_in_hash(&h5));
        assert_eq!(vec![(124, "f"), (125, "f"), (126, "f"), (127, "f")], index_of_lowest_occuring_char_in_hash(&h6));
        assert_eq!(vec![(124, "e"), (125, "e"), (126, "e"), (127, "e")], index_of_lowest_occuring_char_in_hash(&h7));


    }

    #[test]
    fn sha3_512_a_name() {
        assert_eq!(hash_name("AB"), "fcc802621fee9efe4d8ee032d886f75431edb29d480e945d8f0efb1c0ad419bf9b652fca1fa1f5af0f5b4a74f76a6e86b00dbfbec7dcf00e3f4ef34840e9b720");
        assert_eq!(hash_name("2"), "564e1971233e098c26d412f2d4e652742355e616fed8ba88fc9750f869aac1c29cb944175c374a7b6769989aa7a4216198ee12f53bf7827850dfe28540587a97");

        assert_eq!(hash_name("ceo1q0te4aj3u2llwl4mxuxnjm9skj897hncanvgcnz0gf3x57ap6h7gk4dw8nv"), "768ade3da083187a1028dccea3fe7e738c76be4c2ef3fd54bfcfd63f67b34fd588698057a3165b941bbe77355541120c7933efc854ffea0dbb80fcfd7f068a4c");
}

    #[test]
    fn struct_encode_decode() {
        let interest = "interest";
        let encoded = bincode::serialize(&interest).unwrap();
        let decoded: &str = bincode::deserialize(&encoded).unwrap();
        assert_eq!(interest, decoded);
    }

    #[test]
    fn encode_length() {
        let target = "a".to_string();
        let encoded: Vec<u8> = bincode::serialize(&target).unwrap();
        assert_eq!(encoded.len(), 9);
    }
}

