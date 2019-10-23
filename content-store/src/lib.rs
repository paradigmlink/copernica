
pub mod fs;
pub use crate::{fs::Fs};//, tcp::Tcp};
use packets::{mk_data, Packet};

pub trait ContentStore {
    fn has_data(&self, sdri: &Vec<Vec<u16>>) -> Option<Packet>;

    fn box_clone(&self) -> Box::<dyn ContentStore>;
}

impl Clone for Box<dyn ContentStore> {
    fn clone(&self) -> Box<dyn ContentStore> {
        self.box_clone()
    }
}
