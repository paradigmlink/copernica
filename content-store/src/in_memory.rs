
use crate::{ContentStore};
use packets::{mk_data, Packet};

#[derive(Debug, Clone)]
pub struct InMemory {
}

impl InMemory {
    pub fn new() -> Box<InMemory> {
        Box::new(InMemory {
        })
    }

}

impl ContentStore for InMemory {
    fn has_data(&self, sdri: &Vec<Vec<u16>>) -> Option<Packet> {
        None
    }

    fn box_clone(&self) -> Box<dyn ContentStore> {
        Box::new((*self).clone())
    }

}
