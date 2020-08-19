use {
    std::{
        fmt,
    },
    sha3::{
        Digest,
        Sha3_512,
    },
    borsh::{BorshSerialize, BorshDeserialize},
    anyhow::{Result},
    crate::{
        constants,
    }
};

type BFI = [u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize]; // Bloom Filter Index

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
// how to implement hierarchical routing...
// it should be done at node level
// if more than 1 link has an h3 then start route on h2
// if more than 2 links have h2 then route on h1... think about this for a while.
pub struct HBFI { // Hierarchical Bloom Filter Index
    //pub h3: BFI,  // level 3 hierarchy - most coarse
    //pub h2: BFI,  // level 2 hierarchy - comme ci, comme ça
    pub h1: BFI,  // level 1 hierarchy - most fine
    pub id: BFI,  // publisher id
    pub os: u64,  // offset into h1 level of data
}

impl HBFI {
    pub fn new(h1: &str, id: &str) -> Result<HBFI> {
        Ok(HBFI {
            h1: bloom_filter_index(h1)?,
            id: bloom_filter_index(id)?,
            os: 0,
        })
    }
    pub fn offset(mut self, os: u64) -> Self {
        self.os = os;
        self
    }

}

impl fmt::Debug for HBFI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            HBFI { h1, id, os } =>  write!(f, "{:?}::{:?}::{:?}", h1, id, os),
        }
    }
}

fn bloom_filter_index(s: &str) -> Result<[u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize]> {
    use std::str;
    let mut hasher = Sha3_512::new();
    hasher.input(s.as_bytes());
    let hash = hasher.result();
    let mut bloom_filter_index_array: BFI = [0; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize];
    let mut count = 0;
    for n in 0..constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH {
        let mut hasher = Sha3_512::new();
        hasher.input(format!("{:x}{}", hash, n));
        let hs = format!("{:x}", hasher.result());
        let subs = hs.as_bytes()
            .chunks(16)
            .map(str::from_utf8)
            .collect::<Result<Vec<&str>, _>>()?;
        let mut index: u64 = 0;
        for sub in subs {
            let o = u64::from_str_radix(&sub, 16)?;
            index = (index + o) % constants::BLOOM_FILTER_LENGTH;
        }
        bloom_filter_index_array[count] = index as u16;
        count += 1;
    }
    Ok(bloom_filter_index_array)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter_index() {
        let actual = bloom_filter_index("9".into()).unwrap();
        let expected: [u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize] = [4804, 63297, 3290, 20147, 12703, 41640, 34712, 48343];
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
}

