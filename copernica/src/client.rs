extern crate packets;

use {
    client::{CopernicaRequestor},
    log::{trace},
    env_logger,
};

fn main() {
    env_logger::init();
    trace!("copernica client started");
    let requestor = CopernicaRequestor::new("127.0.0.1:8070".into(), "127.0.0.1:8071".into());
    let request1 = "hello1".to_string();
    let response1 = requestor.request(request1.clone());
    trace!("sending request: {}, got response: {:?}", request1, response1);
//    let request2 = "hello2".to_string();
//    let response2 = requestor.request(request2.clone());
//    trace!("sending request: {}, got response: {:?}", request2, response2);
}
