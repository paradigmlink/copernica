use {
    copernica_common::{LinkId, InterLinkPacket, HBFI},
    copernica_protocols::{FTP, Protocol},
    std::{thread},
    crossbeam_channel::{Sender, Receiver, unbounded},
    keynesis::{PrivateIdentity},
    //sled::{Db, Event},
    anyhow::{Result},
};

#[derive(Clone, Debug)]
pub enum FTPCommands {
    RequestFileList(HBFI),
    ResponseFileList(Option<Vec<String>>),
    RequestFile(HBFI, String),
    ResponseFile(Option<Vec<u8>>),
}

pub struct FTPService {
    link_id: Option<LinkId>,
    p2c_tx: Option<Sender<FTPCommands>>,
    c2p_rx: Option<Receiver<FTPCommands>>,
    db: sled::Db,
    protocol: FTP,
    response_sid: PrivateIdentity,
}

impl FTPService {
    pub fn new(db: sled::Db, response_sid: PrivateIdentity) -> Self {
        let protocol: FTP = Protocol::new(db.clone(), response_sid.clone());
        Self {
            link_id: None,
            p2c_tx: None,
            c2p_rx: None,
            db,
            protocol,
            response_sid,
        }
    }

    pub fn peer_with_link(
        &mut self,
        link_id: LinkId,
    ) -> Result<(Sender<InterLinkPacket>, Receiver<InterLinkPacket>)> {
        self.link_id = Some(link_id.clone());
        Ok(self.protocol.peer(link_id)?)
    }

    pub fn peer_with_client(&mut self)
    -> Result<(Sender<FTPCommands>, Receiver<FTPCommands>)> {
        let (c2p_tx, c2p_rx) = unbounded::<FTPCommands>();
        let (p2c_tx, p2c_rx) = unbounded::<FTPCommands>();
        self.p2c_tx = Some(p2c_tx);
        self.c2p_rx = Some(c2p_rx);
        Ok((c2p_tx, p2c_rx))
    }

    pub fn run(&mut self) -> Result<()>{
        let _rs = self.db.clone();
        let p2c_tx = self.p2c_tx.clone();
        let c2p_rx = self.c2p_rx.clone();
        let link_id = self.link_id.clone();
        let mut protocol = self.protocol.clone();
        protocol.run()?;
        thread::spawn(move || {
            if let (Some(c2p_rx), Some(p2c_tx), Some(_link_id)) = (c2p_rx, p2c_tx, link_id) {
                loop {
                    if let Ok(command) = c2p_rx.recv() {
                        match command {
                            FTPCommands::RequestFileList(hbfi) => {
                                let files: Vec<String> = protocol.file_names(hbfi,)?;
                                p2c_tx.send(FTPCommands::ResponseFileList(Some(files.clone())))?;
                            },
                            FTPCommands::RequestFile(hbfi, name) => {
                                let file: Vec<u8> = protocol.file(hbfi, name)?;
                                p2c_tx.send(FTPCommands::ResponseFile(Some(file)))?;
                            },
                            _ => {}
                        }
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }

}
