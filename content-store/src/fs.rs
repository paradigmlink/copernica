
use crate::{ContentStore};
use packets::{mk_data, Packet};

#[derive(Debug, Clone)]
pub struct Fs {
}

impl Fs {
    pub fn new() -> Box<Fs> {
        Box::new(Fs {
        })
    }

}

impl ContentStore for Fs {
    fn has_data(&self, sdri: &Vec<Vec<u16>>) -> Option<Packet> {
        None
    }

    fn box_clone(&self) -> Box<dyn ContentStore> {
        Box::new((*self).clone())
    }

}
