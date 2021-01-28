use {
    copernica_common::{LinkId, NarrowWaistPacket, LinkPacket, InterLinkPacket, HBFI, serialization::*},
    crate::{Manifest, FileManifest, Protocol},
    copernica_identity::{PrivateIdentity},
    crossbeam_channel::{ Sender, Receiver },
    sled::{Db},
    bincode,
    anyhow::{Result, anyhow},
    std::{thread},
    log::{debug},
};
#[derive(Clone)]
pub struct FTP {
    link_id: Option<LinkId>,
    rs: Db,
    l2p_rx: Option<Receiver<InterLinkPacket>>,
    p2l_tx: Option<Sender<InterLinkPacket>>,
    response_sid: PrivateIdentity,
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
    fn new(rs: Db, response_sid: PrivateIdentity) -> FTP {
        FTP {
            response_sid,
            link_id: None,
            l2p_rx: None,
            p2l_tx: None,
            rs,
        }
    }
    fn run(&mut self) -> Result<()> {
        let rs = self.response_store();
        let l2p_rx = self.get_l2p_rx();
        let p2l_tx = self.get_p2l_tx();
        let link_id = self.get_link_id();
        let response_sid = self.get_response_sid();
        thread::spawn(move || {
            if let (Some(l2p_rx), Some(p2l_tx), Some(link_id)) = (l2p_rx, p2l_tx, link_id) {
                loop {
                    if let Ok(ilp) = l2p_rx.recv() {
                        debug!("\t\t|  link-to-protocol");
                        let nw: NarrowWaistPacket = ilp.narrow_waist();
                        match nw.clone() {
                            NarrowWaistPacket::Request { hbfi, .. } => {
                                let (_, hbfi_s) = serialize_hbfi(&hbfi)?;
                                if rs.contains_key(hbfi_s.clone())? {
                                    let nw = rs.get(hbfi_s)?;
                                    match nw {
                                        Some(nw) => {
                                            debug!("\t\t|  RESPONSE PACKET FOUND");
                                            let nw = deserialize_narrow_waist_packet(&nw.to_vec())?;
                                            let lp = LinkPacket::new(link_id.reply_to()?, nw);
                                            let ilp = InterLinkPacket::new(link_id.clone(), lp);
                                            debug!("\t\t|  protocol-to-link");
                                            p2l_tx.send(ilp)?;
                                        },
                                        None => {},
                                    }
                                } else {
                                    let hbfi_ctr = hbfi.cleartext_repr();
                                    let (_, hbfi_ctr) = serialize_hbfi(&hbfi_ctr)?;
                                    if rs.contains_key(hbfi_ctr.clone())? {
                                        let nw = rs.get(hbfi_ctr)?;
                                        match nw {
                                            Some(nw) => {
                                                match hbfi.request_pid {
                                                    Some(request_pid) => {
                                                        debug!("\t\t|  RESPONSE PACKET FOUND ENCRYPT IT");
                                                        let nw = deserialize_narrow_waist_packet(&nw.to_vec())?;
                                                        let nw = nw.encrypt_for(request_pid, response_sid.clone())?;
                                                        let lp = LinkPacket::new(link_id.reply_to()?, nw);
                                                        let ilp = InterLinkPacket::new(link_id.clone(), lp);
                                                        debug!("\t\t|  protocol-to-link");
                                                        p2l_tx.send(ilp.clone())?;
                                                    },
                                                    None => {
                                                        debug!("\t\t|  RESPONSE PACKET FOUND CLEARTEXT IT");
                                                        let nw = deserialize_narrow_waist_packet(&nw.to_vec())?;
                                                        let lp = LinkPacket::new(link_id.reply_to()?, nw);
                                                        let ilp = InterLinkPacket::new(link_id.clone(), lp);
                                                        debug!("\t\t|  protocol-to-link");
                                                        p2l_tx.send(ilp)?;
                                                    },
                                                }
                                            },
                                            None => {},
                                        }
                                    };
                                }
                            },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                debug!("\t\t|  RESPONSE PACKET ARRIVED");
                                let (_, hbfi_s) = serialize_hbfi(&hbfi)?;
                                let (_, nw_s) = serialize_narrow_waist_packet(&nw)?;
                                rs.insert(hbfi_s, nw_s)?;
                            },
                        }
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
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
    fn get_response_sid(&mut self) -> PrivateIdentity {
        self.response_sid.clone()
    }
}

