use {
    copernica_common::{NarrowWaistPacket, HBFI, constants},
    std::{
        path::{Path, PathBuf},
        collections::HashMap,
        io::prelude::*,
        fs,
    },
    serde::{Serialize, Deserialize},
    bincode,
    walkdir::{WalkDir},
    anyhow::{Result, anyhow},
    keynesis::{PrivateIdentity},
    //log::{debug},
};

pub type PathsWithSizes = Vec<(PathBuf, u64)>;
pub type PathsWithOffsets = HashMap<String, (u64, u64)>;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Manifest {
    pub start: u64,
    pub end: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FileManifest {
    pub files: PathsWithOffsets,
}

pub struct FilePacker {
    chunk_size: usize,
    src_dir: PathBuf,
    dest_dir: PathBuf,
    hbfi: HBFI,
    response_sid: PrivateIdentity,
    absolute_files_size: PathsWithSizes,
}

impl FilePacker {
    pub fn new(source_dir: &Path, dest_dir: &Path, hbfi: HBFI, response_sid: PrivateIdentity) -> Result<Self> {
        let src_dir = source_dir.clone().to_path_buf();
        let dest_dir = dest_dir.to_path_buf();
        Ok(Self {
            chunk_size: constants::CLEARTEXT_SIZE,
            src_dir,
            dest_dir,
            hbfi,
            response_sid,
            absolute_files_size: directory_structure(&source_dir)?,
        })
    }

    pub fn src_dir(mut self, dir: &Path) -> Self {
        self.src_dir = dir.to_path_buf();
        self
    }

    pub fn dest_dir(mut self, dir: &Path) -> Self {
        self.dest_dir = dir.to_path_buf();
        self
    }

    pub fn chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    pub fn publish(&self) -> Result<()> {
        let rs = sled::open(&self.dest_dir).expect("open");
        let chunk_size = self.chunk_size;
        let mut relative_files_offsets = PathsWithOffsets::new();
        let mut absolute_files_offsets = PathsWithOffsets::new();
        let mut total_offset: u64 = 1; // 0 is used for manifest, always
        for (entry, size) in &self.absolute_files_size {
            let (start, end, offset) = offset(total_offset, *size, chunk_size as u64)?;
            total_offset += offset;
            let stripped_entry = entry.strip_prefix(self.src_dir.clone())?;
            relative_files_offsets.insert(stripped_entry.to_str().unwrap().to_string(), (start, end));
            absolute_files_offsets.insert(entry.to_str().unwrap().to_string(), (start, end));
        }
        let file_manifest = FileManifest { files: relative_files_offsets };
        let file_manifest: Vec<u8> = bincode::serialize(&file_manifest)?;
        let file_manifest_size = file_manifest.len() as u64;
        let (file_manifest_start, file_manifest_end, file_manifest_offset)= offset(total_offset, file_manifest_size, chunk_size as u64)?;
        total_offset += file_manifest_offset;

        // this concludes the calculation of the total size and file chunk sizes.

        let hbfi = self.hbfi.clone();
        let hbfi_s: Vec<u8> = bincode::serialize(&hbfi)?;

        let mut current_offset: u64 = 1;
        for (file_path, (start, end)) in absolute_files_offsets {
            if end > 0 {
                let mut file = fs::File::open(&file_path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                let file_chunks = buffer.chunks(chunk_size as usize);
                let mut counter = start;
                for file_chunk in file_chunks {
                    let hbfi = hbfi.clone().offset(counter);
                    let hbfi_s: Vec<u8> = bincode::serialize(&hbfi)?;
                    // response_sid: PrivateIdentity, hbfi: HBFI, data: Vec<u8>, offset: u64, total: u64
                    let resp = NarrowWaistPacket::response(self.response_sid.clone(), hbfi.clone(), file_chunk.to_vec(), counter, total_offset)?;
                    //let resp = create_response(hbfi.clone(), file_chunk, counter, total_offset)?;
                    let resp: Vec<u8> = bincode::serialize(&resp)?;
                    rs.insert(&hbfi_s, resp)?;
                    current_offset += 1;
                    counter += 1;
                }
            }
        }

        let file_manifest_chunks = file_manifest.chunks(chunk_size as usize);
        for file_manifest_chunk in file_manifest_chunks {
            let hbfi = hbfi.clone().offset(current_offset);
            let hbfi_s: Vec<u8> = bincode::serialize(&hbfi)?;
            let resp = NarrowWaistPacket::response(self.response_sid.clone(), hbfi.clone(), file_manifest_chunk.to_vec(), current_offset, total_offset)?;
            //let resp = create_response(hbfi.clone(), file_manifest_chunk, current_offset, total_offset)?;
            let resp: Vec<u8> = bincode::serialize(&resp)?;
            rs.insert(&hbfi_s, resp)?;
            current_offset += 1;
        }

        let manifest = Manifest { start: file_manifest_start, end: file_manifest_end };
        let manifest_s: Vec<u8> = bincode::serialize(&manifest)?;
        let resp = NarrowWaistPacket::response(self.response_sid.clone(), hbfi.clone(), manifest_s.to_vec(), 0, total_offset)?;
        //let resp = create_response(hbfi.clone(), &manifest_s, 0, total_offset)?;
        let resp: Vec<u8> = bincode::serialize(&resp)?;
        rs.insert(hbfi_s, resp)?;

        Ok(())
    }
}
/*
fn create_response(hbfi: HBFI, chunk: &[u8], offset: u64, total_offset: u64) -> Result<NarrowWaistPacket> {
    let mut data = [0; constants::FRAGMENT_SIZE as usize];
    let len = chunk.len() as u16;
    if len < constants::FRAGMENT_SIZE as u16 {
        let padded_chunk = [chunk, &vec![0; (constants::FRAGMENT_SIZE as u16 - len) as usize][..]].concat();
        data.copy_from_slice(&padded_chunk);
    } else {
        data.copy_from_slice(&*chunk);
    }
    let data = Data { len: len, data: data };
    Ok(NarrowWaistPacket::Response { hbfi: hbfi.clone(), data, offset: offset, total: total_offset })
}
*/
fn offset(current_offset: u64, size: u64, chunk_size: u64) -> Result<(u64, u64, u64)> {
    //return format = (start address, end address, offset count)
    match size {
        s if s == 0 => {
            return Ok((current_offset, 0, 0)); // current_offset, 0 indicates a file size of 0. clients should disregard a file read totally
        },
        s if (s > 0) && (s <= chunk_size) => {
            return Ok((current_offset, current_offset, 1));
        },
        s if (s > 0) && (s > chunk_size) => {
            let remainder = size % chunk_size;
            let mut offset: f64 = (size / chunk_size) as f64;
            if remainder > 0 {
                offset += 1.0;
            }
            return Ok((current_offset, current_offset+(offset as u64)-1, offset as u64))
        }
        _ => Err(anyhow!("The size of the offset is not covered"))
    }
}

fn directory_structure(dir: &Path) -> Result<PathsWithSizes> {
    let mut files = PathsWithSizes::new();
    for entry in WalkDir::new(dir) {
        let entry = entry.unwrap();
        let path = entry.path();
        let metadata = fs::metadata(&path).unwrap();
        if path.is_dir() == false {
            files.push((path.to_path_buf(), metadata.len()));
        }
    }
    Ok(files)
}
