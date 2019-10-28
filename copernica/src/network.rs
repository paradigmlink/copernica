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
//    packets::{response},
    futures::executor::{ThreadPool},
    std::thread,
};

fn main() {
    env_logger::init();
    trace!("copernica started");
    let client = "127.0.0.1:8070";
    let node1 = "127.0.0.1:8071";
    let node2 = "127.0.0.1:8072";
    let node3 = "127.0.0.1:8073";

    thread::spawn( move || {
        let mut executor = ThreadPool::new().expect("Failed to create threadpool");
        let mut r = Router::new(executor.clone());
        let f1 = Udp::new(node1.clone().into(), client.clone().into());
        let f2 = Udp::new(node2.clone().into(), node3.clone().into());
        r.add_face(f1);
        r.add_face(f2);
        executor.run(r.run())
    });

    thread::park();

}
