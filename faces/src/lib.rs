
mod sparse_distributed_representation;
pub mod udp;
//pub mod tcp;
pub use crate::{udp::Udp};//, tcp::Tcp};

use {
    packets::{Packet},
    futures::executor::ThreadPool,
    crossbeam_channel::{
        unbounded, Sender
    },
};

pub trait Face {
    fn set_id(&mut self, face_id: usize);
    // router uses these
    fn send_interest_downstream(&mut self, interest: Packet);
    fn send_data_upstream(&mut self, data: Packet);

    fn create_pending_interest(&mut self, sdri: &Vec<Vec<u16>>);
    fn contains_pending_interest(&mut self, sdri: &Vec<Vec<u16>>) -> u8;
    fn delete_pending_interest(&mut self, sdri: &Vec<Vec<u16>>);
    fn pending_interest_decoherence(&mut self) -> u8;
    fn partially_forget_pending_interests(&mut self);

    fn create_forwarding_hint(&mut self, sdri: &Vec<Vec<u16>>);
    fn contains_forwarding_hint(&mut self, sdri: &Vec<Vec<u16>>) -> u8;
    fn forwarding_hint_decoherence(&mut self) -> u8;
    fn partially_forget_forwarding_hints(&mut self);

    // application uses these
    //fn try_interest(&self) -> Option<Packet>;
    //fn interest(&self) -> Option<Packet>;


    fn box_clone(&self) -> Box::<dyn Face>;
    fn receive_upstream_interest_or_downstream_data(&mut self, face_id: usize, spawner: ThreadPool, packet_sender: Sender<(usize, Packet)>);
    fn print_pi(&self);
    fn print_fh(&self);
}

impl Clone for Box<dyn Face> {
    fn clone(&self) -> Box<dyn Face> {
        self.box_clone()
    }
}
