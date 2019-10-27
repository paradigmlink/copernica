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
    futures::executor::{ThreadPool},
    std::thread,
};

fn main() {
    env_logger::init();
    trace!("copernica started");
    thread::spawn( move || {
        let mut executor0 = ThreadPool::new().expect("Failed to create threadpool");
        let mut r0 = Router::new(executor0.clone());
        let f1 = Udp::new("127.0.0.1:8090".to_string(), "127.0.0.1:8091".to_string());
        let f2 = Udp::new("127.0.0.1:8092".to_string(), "127.0.0.1:8093".to_string());
        let f3 = Udp::new("127.0.0.1:8094".to_string(), "127.0.0.1:8095".to_string());
        r0.add_face(f1);
        r0.add_face(f2);
        r0.add_face(f3);
        executor0.run(r0.run())
    });

    thread::spawn( move || {
        let mut executor1 = ThreadPool::new().expect("Failed to create threadpool");
        let mut r1 = Router::new(executor1.clone());
        let f4 = Udp::new("127.0.0.1:8096".to_string(), "127.0.0.1:8097".to_string());
        let f5 = Udp::new("127.0.0.1:8098".to_string(), "127.0.0.1:8099".to_string());
        r1.add_face(f4);
        r1.add_face(f5);
        executor1.run(r1.run())
    });

    thread::park();

}
