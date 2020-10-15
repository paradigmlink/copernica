use {
    crate::{Service, DropHookFn},
    copernica_common::{LinkId, InterLinkPacket,},
    crossbeam_channel::{ Sender, Receiver },
    sled::{Db},
};


pub struct RelayNode {
    link_id: Option<LinkId>,
    rs: Db,
    l2s_rx: Option<Receiver<InterLinkPacket>>,
    s2l_tx: Option<Sender<InterLinkPacket>>,
    drop_hook: DropHookFn
}

impl<'a> Service<'a> for RelayNode {
    fn new(rs: Db, drop_hook: DropHookFn) -> RelayNode {
        RelayNode {
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

impl Drop for RelayNode {
    fn drop(&mut self) {
        &(self.drop_hook)();
    }
}
