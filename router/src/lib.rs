extern crate lru_cache;

use lru_cache::LruCache;
use packets::{Interest, Data};
use faces::{Face};

pub struct Router {
    cs: LruCache<String, String>,
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

    #[test]
    fn interest_router_data_and_back_again() {
        let irouter = Router::new();
        let router = Router::new();
        let drouter = Router::new();
    }
}
