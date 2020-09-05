use {
    copernica::{HBFI, Link, InterLinkPacket},
    crate::{
        Requestor, Manifest, FileManifest,
    },
    crossbeam_channel::{ Sender },
    sled::{Db},
    borsh::{BorshDeserialize},
    anyhow::{Result, anyhow},
    log::{debug},
};

#[derive(Clone)]
pub struct FileSharer {
    link: Option<Link>,
    rs: Db,
    sender: Option<Sender<InterLinkPacket>>,
}

impl<'a> FileSharer {
    pub fn manifest(&mut self, hbfi: HBFI) -> Result<Manifest> {
        let hbfi = hbfi.clone().offset(0);
        debug!("File Sharer to Requestor:\t{:?}", hbfi);
        let manifest = self.get(hbfi, 0, 0)?;
        Ok(Manifest::try_from_slice(&manifest)?)
    }
    pub fn file_manifest(&mut self, hbfi: HBFI) -> Result<FileManifest> {
        let manifest: Manifest = self.manifest(hbfi.clone())?;
        let file_manifest = self.get(hbfi, manifest.start, manifest.end)?;
        Ok(FileManifest::try_from_slice(&file_manifest)?)
    }
    pub fn file_names(&mut self, hbfi: HBFI) -> Result<Vec<String>> {
        let file_manifest: FileManifest = self.file_manifest(hbfi.clone())?;
        let mut names: Vec<String> = vec![];
        for (path, _) in file_manifest.files {
            names.push(path);
        }
        Ok(names)
    }
    pub fn file(&mut self, hbfi: HBFI, name: String) -> Result<Vec<u8>> {
        let file_manifest: FileManifest = self.file_manifest(hbfi.clone())?;
        if let Some((start, end)) = file_manifest.files.get(&name) {
            let file = self.get(hbfi.clone(), *start, *end)?;
            return Ok(file);
        }
        return Err(anyhow!("File not present"))
    }
}

impl<'a> Requestor<'a> for FileSharer {
    fn new(rs: Db) -> FileSharer {
        FileSharer {
            link: None,
            sender: None,
            rs,
        }
    }
    fn response_store(&self) -> Db {
        self.rs.clone()
    }
    fn set_sender(&mut self, sender: Option<Sender<InterLinkPacket>>) {
        self.sender = sender;
    }
    fn get_sender(&mut self) -> Option<Sender<InterLinkPacket>> {
        self.sender.clone()
    }
    fn get_link(&mut self) -> Option<Link> {
        self.link.clone()
    }
    fn set_link(&mut self, link: Link) {
        self.link = Some(link);
    }
}

