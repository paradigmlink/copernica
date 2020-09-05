use {
    copernica::{Link, InterLinkPacket},
    crate::{
        Requestor
    },
    crossbeam_channel::{ Sender },
    sled::{Db},
};


#[derive(Clone)]
pub struct RelayNode {
    link: Option<Link>,
    rs: Db,
    sender: Option<Sender<InterLinkPacket>>,
}

impl<'a> Requestor<'a> for RelayNode {
    fn new(rs: Db) -> RelayNode {
        RelayNode {
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
