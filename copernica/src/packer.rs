use {
    borsh::{BorshSerialize, BorshDeserialize},
    std::{
        path::{Path, PathBuf},
        io::prelude::*,
        fs,
    },
    walkdir::{WalkDir},
    anyhow::{Result},
    //sled,
    crate::{
        narrow_waist::{mk_response_packet},
        //sdri::{Sdri},
        constants,
    },
};

type FileNameAndSize = Vec<(PathBuf, u64)>;
type FileNameAndOffset = Vec<(String, u64)>;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
struct ResponseManifest {
    app_type: u64,
    chunk_size: u16,
    offset: u64,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
struct ResponseFileManifest {
    files: FileNameAndOffset,
}

pub struct Packer {
    chunk_size: u16,
    src_dir: PathBuf,
    dest_dir: PathBuf,
    name: String,
    files: FileNameAndSize,
}

impl Packer {
    pub fn new(source_dir: &Path, dest_dir: &Path) -> Result<Self> {
        let src_dir = source_dir.clone().to_path_buf();
        let dest_dir = dest_dir.to_path_buf();
        Ok(Self {
            chunk_size: constants::FRAGMENT_SIZE,
            src_dir,
            dest_dir,
            name: source_dir.clone().file_name().unwrap().to_str().unwrap().to_string(),
            files: directory_structure(&source_dir)?,
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

    pub fn publish(&self) -> Result<()> {
        let response = sled::open(&self.dest_dir).expect("open");
        let mut files_offset = FileNameAndOffset::new();
        let mut files_size = FileNameAndSize::new();
        let mut files_offset_stripped = FileNameAndOffset::new();
        let mut total_offset: u64 = 0;
        for (entry, size) in &self.files {
            let entry_stripped = entry.strip_prefix(self.src_dir.clone())?;
            let mut offset = self.offset(size);
            if offset != 0 {
                offset -= 1;
            }
            total_offset += offset;
            files_offset_stripped.push((entry_stripped.to_str().unwrap().to_string(), offset));
            files_offset.push((entry.to_str().unwrap().to_string(), offset));
            files_size.push((entry.to_path_buf(), *size));
        }
        let file_manifest = ResponseFileManifest { files: files_offset_stripped };
        let file_manifest_ser = file_manifest.try_to_vec()?;
        let file_manifest_size = file_manifest_ser.len() as u64;
        let file_manifest_offset = self.offset(&file_manifest_size);
        total_offset += file_manifest_offset as u64;

        let manifest = ResponseManifest{ app_type: 0, chunk_size: self.chunk_size, offset: file_manifest_offset };
        let manifest_ser = manifest.try_to_vec()?;

        response.insert("0", mk_response_packet(self.name.clone(), manifest_ser.clone(), 0, total_offset).try_to_vec()?)?;

        let mut current_offset: u64 = 1; // start at 1 cause the manifest, which is always length 1 and in element 0.
        let file_manifest_chunks = file_manifest_ser.chunks(self.chunk_size as usize);
        for chunk in file_manifest_chunks {
            response.insert(&current_offset.to_string(),
                mk_response_packet(self.name.clone(), chunk.clone().to_vec(), current_offset, total_offset).try_to_vec()?)?;
            current_offset += 1;
        }

        for (file_path, offset) in files_offset {
            if offset > 0 {
                let mut file = fs::File::open(&file_path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                let file_chunks = buffer.chunks(self.chunk_size as usize);
                for chunk in file_chunks {
                    response.insert(&current_offset.to_string(),
                        mk_response_packet(self.name.clone(), chunk.clone().to_vec(), current_offset, total_offset).try_to_vec()?)?;
                    current_offset += 1;
                }
            }
        }
        Ok(())
    }

    fn offset(&self, size: &u64) -> u64 {
        let mut offset: f64 = 0.0;
        if size < &(self.chunk_size as u64) {
            if size != &0 {
                offset += 1.0;
            }
        } else {
            let remainder = size % self.chunk_size as u64;
            offset = (size / self.chunk_size as u64) as f64;
            offset += 1.0; // needs to match with the above buffer.chunks count
            if (remainder > 0) && (remainder < self.chunk_size.into()) {
                offset += 1.0;
            }
        }
        offset as u64
    }
}

fn directory_structure(dir: &Path) -> Result<FileNameAndSize> {
    let mut files = FileNameAndSize::new();
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
