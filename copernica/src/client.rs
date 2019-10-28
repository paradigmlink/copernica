extern crate packets;

use {
    client::{CopernicaRequestor},
    log::{trace},
    env_logger,
};

fn main() {
    env_logger::init();
    trace!("copernica client started");
    let requestor = CopernicaRequestor::new("127.0.0.1:8091".into(), "127.0.0.1:8090".into());
    let request = "hello1".to_string();
    let response = requestor.request(request.clone());
    trace!("sending request: {}, got response: {:?}", request, response);
}
