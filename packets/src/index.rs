use sha3::{Digest, Sha3_512};
use crate::Sdri;

const HEX : [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f"];

fn hash_name(s: &str) -> String {
    let mut hasher = Sha3_512::new();
    hasher.input(s.as_bytes());
    format!("{:x}", hasher.result())
}

fn index_of_lowest_occuring_char_in_hash<'a>( hash: &'a str) -> Vec<(u16, &'a str)> {
    let mut old: Vec<(usize, &str)> = vec![(0,""); 15];
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
    u.iter().map(|(x, y)| (x * 16 + (u16::from_str_radix(y, 16).unwrap()))).collect::<Vec<u16>>()
}

fn name_sparsity(s: &str) -> Vec<(u16)> {
    gen_2048_sparsity(index_of_lowest_occuring_char_in_hash(&hash_name(s)))
}

pub fn generate_sdr_index(s: String) -> Sdri {
    let names = s.split('-');
    let names: Vec<&str> = names.collect();
    let mut fh: Sdri = Vec::new();
    for name in names {
        fh.push(name_sparsity(name));
    }
    fh
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate flate2;
    extern crate tar;

    use std::fs;
    use flate2::read::GzDecoder;
    use tar::Archive;
    use std::io::{BufRead, BufReader};
    use bitvec::prelude::*;
    use crate::{Packet, request};

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

    #[test]
    fn test_interest_creation() {
        let interest = request("mozart-topology-data".to_string());
        assert_eq!(
            Packet::Request {
                sdri: vec![vec![542, 1886, 2014], vec![724, 1588, 1700], vec![160, 528, 720, 992]] }
            , interest);
    }

    #[test]
    fn gen_sdr_index() {
        let s = "domain-app-data-stewart-calculus-topology-mozart-Johann Sebastian Bach-abracadabra-abc";
        assert_eq!(
        vec![
            vec![898, 978, 1074, 1394],
            vec![175, 575, 687, 1231, 1567],
            vec![160, 528, 720, 992],
            vec![1126, 1286, 1542, 1654],
            vec![355, 387, 419, 675, 1763],
            vec![724, 1588, 1700],
            vec![542, 1886, 2014],
            vec![1037, 1565, 1773, 1789],
            vec![145, 945, 1153, 1745],
            vec![154, 250, 1210, 1306, 1770]]
        , generate_sdr_index(s.to_string()));
    }

    #[test]
    fn test_creation_of_sparsity_in_a_2048_bit_vector() {
        let h1 = hash_name("AAAL");
        let h2 = hash_name("AB");
        let h3 = hash_name("2");
        assert_eq!(vec![289, 305, 1137, 1377, 1633], gen_2048_sparsity(index_of_lowest_occuring_char_in_hash(&h1)));
        assert_eq!(vec![355, 515, 1795, 1875], gen_2048_sparsity(index_of_lowest_occuring_char_in_hash(&h2)));
        assert_eq!(vec![192, 848, 1808, 1936], gen_2048_sparsity(index_of_lowest_occuring_char_in_hash(&h3)));
    }

    #[test]
    fn find_index_of_char_in_hash() {
        let h1 = hash_name("AAAL");
        let h2 = hash_name("AB");
        let h3 = hash_name("2");
        let h4 = hash_name("AAA");
        assert_eq!(vec![(18, "1"), (19, "1"), (71, "1"), (86, "1"), (102, "1")], index_of_lowest_occuring_char_in_hash(&h1));
        assert_eq!(vec![(22, "3"), (32, "3"), (112, "3"), (117, "3")], index_of_lowest_occuring_char_in_hash(&h2));
        assert_eq!(vec![(12, "0"), (53, "0"), (113, "0"), (121, "0")], index_of_lowest_occuring_char_in_hash(&h3));
        assert_eq!(vec![(3, "c"), (16, "c"), (65, "c")], index_of_lowest_occuring_char_in_hash(&h4));

    }

    #[test]
    fn sha3_512_a_name() {
        assert_eq!(hash_name("AB"), "fcc802621fee9efe4d8ee032d886f75431edb29d480e945d8f0efb1c0ad419bf9b652fca1fa1f5af0f5b4a74f76a6e86b00dbfbec7dcf00e3f4ef34840e9b720");
        assert_eq!(hash_name("2"), "564e1971233e098c26d412f2d4e652742355e616fed8ba88fc9750f869aac1c29cb944175c374a7b6769989aa7a4216198ee12f53bf7827850dfe28540587a97");

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
