extern crate packets;
use std::env;

use {
    copernica_lib::{CopernicaRequestor},
    log::{trace},
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let remote: String = args[1].clone().to_string();
    let request: String = args[2].clone().to_string();
    trace!("copernica client started");
    let mut cr = CopernicaRequestor::new(remote);
    let response = cr.request(vec![request.clone()], 50);
    trace!("sending request: {}, got response: {:?}", request, response);
}
