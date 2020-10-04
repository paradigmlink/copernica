use {
    copernica_core::{LinkId, InterLinkPacket},
    crate::{
        CopernicaApp, DropHookFn
    },
    crossbeam_channel::{ Sender },
    sled::{Db},
};


pub struct RelayNode {
    link_id: Option<LinkId>,
    rs: Db,
    sender: Option<Sender<InterLinkPacket>>,
    drop_hook: DropHookFn
}

impl<'a> CopernicaApp<'a> for RelayNode {
    fn new(rs: Db, drop_hook: DropHookFn) -> RelayNode {
        RelayNode {
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

impl Drop for RelayNode {
    fn drop(&mut self) {
        &(self.drop_hook)();
    }
}
