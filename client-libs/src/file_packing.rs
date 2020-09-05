use {
    copernica::{NarrowWaist, Data, HBFI, copernica_constants},
    std::{
        path::{Path, PathBuf},
        collections::HashMap,
        io::prelude::*,
        fs,
    },
    borsh::{BorshSerialize, BorshDeserialize},
    walkdir::{WalkDir},
    anyhow::{Result, anyhow},
    //log::{debug},
};

pub type PathsWithSizes = Vec<(PathBuf, u64)>;
pub type PathsWithOffsets = HashMap<String, (u64, u64)>;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Manifest {
    pub start: u64,
    pub end: u64,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct FileManifest {
    pub files: PathsWithOffsets,
}

pub struct FilePacker {
    chunk_size: u16,
    src_dir: PathBuf,
    dest_dir: PathBuf,
    name: String,
    id: String,
    absolute_files_size: PathsWithSizes,
}

impl FilePacker {
    pub fn new(source_dir: &Path, dest_dir: &Path, name: String, id: String) -> Result<Self> {
        let src_dir = source_dir.clone().to_path_buf();
        let dest_dir = dest_dir.to_path_buf();
        Ok(Self {
            chunk_size: copernica_constants::FRAGMENT_SIZE,
            src_dir,
            dest_dir,
            name,
            id,
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

    pub fn chunk_size(mut self, chunk_size: u16) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn id(mut self, id: String) -> Self {
        self.id = id;
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
        let file_manifest = file_manifest.try_to_vec()?;
        let file_manifest_size = file_manifest.len() as u64;
        let (file_manifest_start, file_manifest_end, file_manifest_offset)= offset(total_offset, file_manifest_size, chunk_size as u64)?;
        total_offset += file_manifest_offset;

        // this concludes the calculation of the total size and file chunk sizes.

        let hbfi = HBFI::new(&self.name, &self.id)?;

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
                    let resp = create_response(hbfi.clone(), file_chunk, counter, total_offset)?.try_to_vec()?;
                    rs.insert(&hbfi.try_to_vec()?, resp)?;
                    current_offset += 1;
                    counter += 1;
                }
            }
        }

        let file_manifest_chunks = file_manifest.chunks(chunk_size as usize);
        for file_manifest_chunk in file_manifest_chunks {
            let hbfi = hbfi.clone().offset(current_offset);
            let resp = create_response(hbfi.clone(), file_manifest_chunk, current_offset, total_offset)?.try_to_vec()?;
            rs.insert(&hbfi.try_to_vec()?, resp)?;
            current_offset += 1;
        }

        let manifest = Manifest { start: file_manifest_start, end: file_manifest_end }.try_to_vec()?;
        let resp = create_response(hbfi.clone(), &manifest, 0, total_offset)?.try_to_vec()?;
        rs.insert(hbfi.try_to_vec()?, resp)?;

        Ok(())
    }
}

fn create_response(hbfi: HBFI, chunk: &[u8], offset: u64, total_offset: u64) -> Result<NarrowWaist> {
    let mut data = [0; copernica_constants::FRAGMENT_SIZE as usize];
    let len = chunk.len() as u16;
    if len < copernica_constants::FRAGMENT_SIZE {
        let padded_chunk = [chunk, &vec![0; (copernica_constants::FRAGMENT_SIZE - len) as usize][..]].concat();
        data.copy_from_slice(&padded_chunk);
    } else {
        data.copy_from_slice(&*chunk);
    }
    let data = Data { len: len, data: data };
    Ok(NarrowWaist::Response { hbfi: hbfi.clone(), data, offset: offset, total: total_offset })
}

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
