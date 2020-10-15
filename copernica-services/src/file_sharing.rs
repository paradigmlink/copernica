use {
    copernica_common::{HBFI, LinkId, InterLinkPacket},
    crate::{Manifest, FileManifest, Service, DropHookFn},
    crossbeam_channel::{ Sender, Receiver },
    sled::{Db},
    borsh::{BorshDeserialize},
    anyhow::{Result, anyhow},
    log::{debug},
};

pub struct FileSharer {
    link_id: Option<LinkId>,
    rs: Db,
    l2s_rx: Option<Receiver<InterLinkPacket>>,
    s2l_tx: Option<Sender<InterLinkPacket>>,
    drop_hook: DropHookFn,
}

impl<'a> FileSharer {
    pub fn manifest(&mut self, hbfi: HBFI) -> Result<Manifest> {
        let hbfi = hbfi.clone().offset(0);
        debug!("File Sharer to Service:\t{:?}", hbfi);
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

impl<'a> Service<'a> for FileSharer {
    fn new(rs: Db, drop_hook: DropHookFn) -> FileSharer {
        FileSharer {
            link_id: None,
            l2s_rx: None,
            s2l_tx: None,
            rs,
            drop_hook,
        }
    }
    fn response_store(&self) -> Db {
        self.rs.clone()
    }
    fn set_l2s_rx(&mut self, r: Receiver<InterLinkPacket>) {
        self.l2s_rx = Some(r);
    }
    fn get_l2s_rx(&mut self) -> Option<Receiver<InterLinkPacket>> {
        self.l2s_rx.clone()
    }
    fn set_s2l_tx(&mut self, s: Sender<InterLinkPacket>) {
        self.s2l_tx = Some(s);
    }
    fn get_s2l_tx(&mut self) -> Option<Sender<InterLinkPacket>> {
        self.s2l_tx.clone()
    }
    fn set_link_id(&mut self, link_id: LinkId) {
        self.link_id = Some(link_id);
    }
    fn get_link_id(&mut self) -> Option<LinkId> {
        self.link_id.clone()
    }
}

