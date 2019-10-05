extern crate lru_cache;

use lru_cache::LruCache;
use packets::{Interest, Data};
use faces::{Face, Mock};
use crossbeam_utils;

#[derive(Clone)]
pub struct Router<'a> {
    faces: Vec<&'a dyn Face>,
    cs: LruCache<String, String>,
    pit: bool,
    fib: bool,
    is_running: bool,
}

impl<'a> Router<'a> {
    pub fn new() -> Self {
        Router {
            faces: Vec::new(),
            cs: LruCache::new(10),
            pit: false,
            fib: false,
            is_running: false,
        }
    }

    pub fn add_face(&mut self, face: &'a dyn Face) {
        self.faces.push(face);
    }

    pub fn run(&mut self) {
        self.is_running = true;
        crossbeam_utils::thread::scope(|_scope| {
            loop {
                match self.faces[0].interest_poll() {
                    Some(i) => self.faces[1].interest_in(i),
                    None => { if self.is_running == true { break } else { continue }},
                }
            }
         });
    }

    pub fn stop(mut self) {
        self.is_running = false;
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::{unbounded};

    #[test]
    fn setup_router_and_ensure_faces_still_operate_does_not_pass_ownership_into_router() {
        let f1: Mock = Face::new();
        let mut router = Router::new();
        router.add_face(&f1);
        let interest = Interest::new("interest".to_string());
        f1.interest_in(interest.clone());
        assert_eq!(interest, f1.interest_poll().unwrap());
    }

    #[test]
    fn test_throughput() {
        let f1: Mock = Face::new();
        let f2: Mock = Face::new();
        let f3: Mock = Face::new();
        let f4: Mock = Face::new();
        let mut r1 = Router::new();
        let mut r2 = Router::new();
        r1.add_face(&f1);
        r1.add_face(&f2);
        r2.add_face(&f3);
        r2.add_face(&f4);
        let interest = Interest::new("interest".to_string());
        let (i_in, i_out) = unbounded();
        f1.interest_in(interest.clone());
        r1.run();
        i_in.send(f2.interest_poll()).unwrap();
        f3.interest_in(i_out.recv().unwrap().unwrap());
        r2.run();
        assert_eq!(interest, f4.interest_poll().unwrap());

        r1.stop();
        r2.stop();
        // i -> f1 r1 f2 -> f3 r2 f4

    }

    #[test]
    fn batch_feed_interests() {
        let is: Vec<Interest> = vec![Interest::new("1".to_string()), Interest::new("2".to_string()), Interest::new("3".to_string())];
        let f1: Mock = Face::new();
        let f2: Mock = Face::new();
        let mut r1 = Router::new();
        r1.add_face(&f1);
        r1.add_face(&f2);
        for i in &is {
            f1.interest_in(i.clone());
        }
        r1.run();
        let mut ois: Vec<Interest> = Vec::new();
        while let Some(i) = f2.interest_poll() {
            ois.push(i);
        }
        assert_eq!(is, ois);
    }
}
