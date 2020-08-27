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
        copernica_constants,
    }
};

type BFI = [u16; copernica_constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize]; // Bloom Filter Index

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

fn bloom_filter_index(s: &str) -> Result<[u16; copernica_constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize]> {
    use std::str;
    let mut hasher = Sha3_512::new();
    hasher.input(s.as_bytes());
    let hash = hasher.result();
    let mut bloom_filter_index_array: BFI = [0; copernica_constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize];
    let mut count = 0;
    for n in 0..copernica_constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH {
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
            index = (index + o) % copernica_constants::BLOOM_FILTER_LENGTH;
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
        let expected: [u16; copernica_constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize] = [4804, 63297, 3290, 20147, 12703, 41640, 34712, 48343];
        assert_eq!(actual, expected);
    }
}

