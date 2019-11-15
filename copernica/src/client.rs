extern crate packets;
use std::env;

use {
    client::{CopernicaRequestor},
    log::{trace},
};

fn main() {
//    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let remote: String = args[1].clone().to_string();
    let request: String = args[2].clone().to_string();
    trace!("copernica client started");
    let mut cr = CopernicaRequestor::new(remote);
    let response = cr.request(vec![request.clone()], 50);
    trace!("sending request: {}, got response: {:?}", request, response);
//    let request2 = "hello2".to_string();
//    let response2 = requestor.request(request2.clone());
//    trace!("sending request: {}, got response: {:?}", request2, response2);
}
