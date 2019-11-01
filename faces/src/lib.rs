mod sparse_distributed_representation;
pub mod udp;
pub use crate::{udp::Udp};

use {
    packets::{Packet},
    futures::executor::ThreadPool,
    crossbeam_channel::{Sender},
};

pub trait Face {
    fn set_id(&mut self, face_id: usize);
    fn get_id(&self) -> usize;
    fn send_request_downstream(&mut self, interest: Packet);
    fn send_response_upstream(&mut self, data: Packet);
    fn receive_upstream_request_or_downstream_response(&mut self, spawner: ThreadPool, packet_sender: Sender<(usize, Packet)>);

    fn create_pending_request(&mut self, sdri: &Vec<Vec<u16>>);
    fn contains_pending_request(&mut self, sdri: &Vec<Vec<u16>>) -> u8;
    fn delete_pending_request(&mut self, sdri: &Vec<Vec<u16>>);
    fn pending_request_decoherence(&mut self) -> u8;
    fn partially_forget_pending_request(&mut self);

    fn create_forwarded_request(&mut self, packet_sdri: &Vec<Vec<u16>>);
    fn contains_forwarded_request(&mut self, request_sdri: &Vec<Vec<u16>>) -> u8;
    fn delete_forwarded_request(&mut self, request_sdri: &Vec<Vec<u16>>);
    fn forwarded_request_decoherence(&mut self) -> u8;
    fn partially_forget_forwarded_request(&mut self);

    fn create_forwarding_hint(&mut self, sdri: &Vec<Vec<u16>>);
    fn contains_forwarding_hint(&mut self, sdri: &Vec<Vec<u16>>) -> u8;
    fn forwarding_hint_decoherence(&mut self) -> u8;
    fn partially_forget_forwarding_hint(&mut self);

    fn box_clone(&self) -> Box::<dyn Face>;
    fn print_pi(&self);
    fn print_fh(&self);
}

impl Clone for Box<dyn Face> {
    fn clone(&self) -> Box<dyn Face> {
        self.box_clone()
    }
}
