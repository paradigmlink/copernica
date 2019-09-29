extern crate lru_cache;

use lru_cache::LruCache;
use packets::{Interest, Data};

pub struct Router {
    cs: LruCache<String, Data>,
    pit: bool,
    fib: bool
}

impl Router {
    pub fn new() -> Self {
        Router {
            cs: LruCache::new(10),
            pit: false,
            fib: false,
        }
    }

    pub fn run(self) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn content_store() {
        let router = Router::new();
        router.run();
        assert_eq!(2 + 2, 4);
    }
}
