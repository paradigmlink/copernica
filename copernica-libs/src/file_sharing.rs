use {
    copernica_core::{HBFI, LinkId, InterLinkPacket},
    crate::{
        CopernicaApp, Manifest, FileManifest, DropHookFn
    },
    crossbeam_channel::{ Sender },
    sled::{Db},
    borsh::{BorshDeserialize},
    anyhow::{Result, anyhow},
    log::{debug},
};

pub struct FileSharer {
    link_id: Option<LinkId>,
    rs: Db,
    sender: Option<Sender<InterLinkPacket>>,
    drop_hook: DropHookFn,
}

impl<'a> FileSharer {
    pub fn manifest(&mut self, hbfi: HBFI) -> Result<Manifest> {
        let hbfi = hbfi.clone().offset(0);
        debug!("File Sharer to CopernicaApp:\t{:?}", hbfi);
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

impl Drop for FileSharer {
    fn drop(&mut self) {
        &(self.drop_hook)();
    }
}

impl<'a> CopernicaApp<'a> for FileSharer {
    fn new(rs: Db, drop_hook: DropHookFn) -> FileSharer {
        FileSharer {
            link_id: None,
            sender: None,
            rs,
            drop_hook,
        }
    }
    fn response_store(&self) -> Db {
        self.rs.clone()
    }
    fn set_app_link_tx(&mut self, sender: Option<Sender<InterLinkPacket>>) {
        self.sender = sender;
    }
    fn get_app_link_tx(&mut self) -> Option<Sender<InterLinkPacket>> {
        self.sender.clone()
    }
    fn get_app_link_id(&mut self) -> Option<LinkId> {
        self.link_id.clone()
    }
    fn set_app_link_id(&mut self, link_id: LinkId) {
        self.link_id = Some(link_id);
    }
}

