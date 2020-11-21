use {
    copernica_common::{HBFI, LinkId, InterLinkPacket},//, NarrowWaistPacket},
    crate::{Manifest, FileManifest, Protocol},
    copernica_identity::{PrivateIdentity},
    crossbeam_channel::{ Sender, Receiver },
    sled::{Db},
    bincode,
    anyhow::{Result, anyhow},
};
#[derive(Clone)]
pub struct FTP {
    link_id: Option<LinkId>,
    rs: Db,
    l2p_rx: Option<Receiver<InterLinkPacket>>,
    p2l_tx: Option<Sender<InterLinkPacket>>,
    request_sid: PrivateIdentity,
}
impl<'a> FTP {
    pub fn manifest(&mut self, hbfi: HBFI) -> Result<Manifest> {
        let hbfi = hbfi.clone().offset(0);
        let manifest = self.get(hbfi.clone(), 0, 0)?;
        let manifest: Manifest = bincode::deserialize(&manifest)?;
        Ok(manifest)
    }
    pub fn file_manifest(&mut self, hbfi: HBFI) -> Result<FileManifest> {
        let manifest: Manifest = self.manifest(hbfi.clone())?;
        let file_manifest = self.get(hbfi, manifest.start, manifest.end)?;
        let file_manifest: FileManifest = bincode::deserialize(&file_manifest)?;
        Ok(file_manifest)
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
impl<'a> Protocol<'a> for FTP {
    fn new(rs: Db, request_sid: PrivateIdentity) -> FTP {
        FTP {
            request_sid,
            link_id: None,
            l2p_rx: None,
            p2l_tx: None,
            rs,
        }
    }
    fn response_store(&self) -> Db {
        self.rs.clone()
    }
    fn set_l2p_rx(&mut self, r: Receiver<InterLinkPacket>) {
        self.l2p_rx = Some(r);
    }
    fn get_l2p_rx(&mut self) -> Option<Receiver<InterLinkPacket>> {
        self.l2p_rx.clone()
    }
    fn set_p2l_tx(&mut self, s: Sender<InterLinkPacket>) {
        self.p2l_tx = Some(s);
    }
    fn get_p2l_tx(&mut self) -> Option<Sender<InterLinkPacket>> {
        self.p2l_tx.clone()
    }
    fn set_link_id(&mut self, link_id: LinkId) {
        self.link_id = Some(link_id);
    }
    fn get_link_id(&mut self) -> Option<LinkId> {
        self.link_id.clone()
    }
    fn get_request_sid(&mut self) -> PrivateIdentity {
        self.request_sid.clone()
    }
}

