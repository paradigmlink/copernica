extern crate bincode;
extern crate faces;
extern crate router;
extern crate futures;
extern crate content_store;

use faces::{Udp};
use router::{Router};
use content_store::{Fs, InMemory};
use futures::executor::ThreadPool;
use futures::future::join;

fn main() {
    let mut executor = ThreadPool::new().expect("Failed to create threadpool");
    let mut r = Router::new();
    let fs = Fs::new();
    let f1 = Udp::new("127.0.0.1:8090".to_string(), "127.0.0.1:8091".to_string());
    let f2 = Udp::new("127.0.0.1:8092".to_string(), "127.0.0.1:8093".to_string());
    let f3 = Udp::new("127.0.0.1:8094".to_string(), "127.0.0.1:8095".to_string());
    r.add_content_store(fs);
    r.add_face(f1);
    r.add_face(f2);
    r.add_face(f3);

    let mut r1 = Router::new();
    let f4 = Udp::new("127.0.0.1:8090".to_string(), "127.0.0.1:8091".to_string());
    r1.add_face(f4);

    let j = join(r.run(executor.clone()), r1.run(executor.clone()));

    executor.run(j);
}
