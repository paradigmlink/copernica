
pub mod fs;
pub mod in_memory;
pub use crate::{fs::Fs, in_memory::InMemory};
use packets::{Packet};
use std::vec::Vec;

pub trait ContentStore {
    fn has_data(&self, sdri: &Vec<Vec<u16>>) -> Option<Packet>;
    fn put_data(&mut self, data: Packet);
    fn box_clone(&self) -> Box::<dyn ContentStore>;
}

impl Clone for Box<dyn ContentStore> {
    fn clone(&self) -> Box<dyn ContentStore> {
        self.box_clone()
    }
}
