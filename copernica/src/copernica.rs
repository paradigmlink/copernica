extern crate bincode;
extern crate faces;
extern crate router;
extern crate futures;
extern crate content_store;
extern crate log;
extern crate env_logger;

use {
    log::{trace},
    faces::{Udp},
    router::{Router},
    packets::{response},
    futures::executor::{ThreadPool},
    std::thread,
};

fn main() {
    env_logger::init();
    trace!("copernica started");
    thread::spawn( move || {
        let mut executor = ThreadPool::new().expect("Failed to create threadpool");
        let mut r = Router::new(executor.clone());
        let f1 = Udp::new("127.0.0.1:8090".to_string(), "127.0.0.1:8091".to_string());
        let f2 = Udp::new("127.0.0.1:8092".to_string(), "127.0.0.1:8093".to_string());
        r.add_face(f1);
        r.add_face(f2);
        executor.run(r.run())
    });

    thread::spawn( move || {
        let mut executor = ThreadPool::new().expect("Failed to create threadpool");
        let mut r = Router::new(executor.clone());
        let f1 = Udp::new("127.0.0.1:8093".to_string(), "127.0.0.1:8092".to_string());
        let f2 = Udp::new("127.0.0.1:8094".to_string(), "127.0.0.1:8095".to_string());
        r.add_face(f1);
        r.add_face(f2);
        executor.run(r.run())
    });

    thread::spawn( move || {
        let mut executor = ThreadPool::new().expect("Failed to create threadpool");
        let mut r = Router::new(executor.clone());
        let f1 = Udp::new("127.0.0.1:8095".to_string(), "127.0.0.1:8094".to_string());
        let f2 = Udp::new("127.0.0.1:8096".to_string(), "127.0.0.1:8097".to_string());
        r.add_face(f1);
        r.add_face(f2);
        executor.run(r.run())
    });

    thread::spawn( move || {
        let mut executor = ThreadPool::new().expect("Failed to create threadpool");
        let mut r = Router::new(executor.clone());
        let f = Udp::new("127.0.0.1:8097".to_string(), "127.0.0.1:8096".to_string());
        r.insert_into_cs(response("hello".to_string(), "hello".to_string().as_bytes().to_vec()));
        r.add_face(f);
        executor.run(r.run())
    });

    thread::park();

}
