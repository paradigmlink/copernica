extern crate lru_cache;

use lru_cache::LruCache;
use packets::{Interest, Data};
use faces::{Face, Mock};

#[derive(Clone)]
pub struct Router<'a> {
    faces: Vec<&'a dyn Face>,
    cs: LruCache<String, String>,
    pit: bool,
    fib: bool
}

impl<'a> Router<'a> {
    pub fn new() -> Self {
        Router {
            faces: Vec::new(),
            cs: LruCache::new(10),
            pit: false,
            fib: false,
        }
    }

    pub fn add_face(&mut self, face: &'a dyn Face) {
        self.faces.push(face);
    }

    pub fn run(self) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interest_router_data_and_back_again() {
        let f1: Mock = Face::new();
        let f2: Mock = Face::new();
        let mut router = Router::new();
        router.add_face(&f1);
        router.add_face(&f2);
    }
}
