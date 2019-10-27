use packets::{Packet};
use std::net::{SocketAddrV4};
use futures::Future;


pub struct CopernicaRequest {
    end_point: SocketAddrV4,
    data_name: String,
}
#[allow(dead_code)]
impl CopernicaRequest {
    fn new(data_name: String, server: String) -> Self {
        CopernicaRequest { data_name: data_name, end_point: server.parse().unwrap() }
    }
    fn request(self) -> impl Future<Output = Packet> + 'static {
        let end_point = self.end_point.clone();
        let data_name = self.data_name.clone();
        async move {
            send(data_name, end_point).await
        }
    }
}

async fn send(name: String, server: SocketAddrV4) -> Packet {
    packets::response(name.clone(), name.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    #[test]
    fn simple_name_request() {
        let hello = "hello".to_string();
        let resp_future = CopernicaRequest::new(hello.clone(), "127.0.0.1:8090".to_string()).request();
        let resp = block_on(resp_future);
        let response_construct = packets::response(hello.clone(), hello.as_bytes().to_vec());
        assert_eq!(response_construct, resp);
    }
}
