use {
    packets::{Packet, generate_sdr_index, response, request},
    bincode::{serialize, deserialize},
    std::net::{SocketAddrV4},
    crossbeam_channel::{Sender, Receiver, unbounded},
    async_std::{
        net::UdpSocket,
        task,
    },
};

pub struct CopernicaRequestor {
    listen_on: SocketAddrV4,
    remote_on: SocketAddrV4,
}

impl CopernicaRequestor {
    pub fn new(listen_on: String, remote_on: String) -> CopernicaRequestor {
        CopernicaRequestor {
            listen_on: listen_on.parse().unwrap(),
            remote_on: remote_on.parse().unwrap(),
        }
    }
    pub fn request(&self, name: String) -> Packet { // -> (String, Packet)
        let remote_on = self.remote_on.clone();
        let (s, r) = unbounded();
        let name = name.clone();
        task::block_on( async {
            let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let packet_ser = serialize(&request(name.clone())).unwrap();
            let _r = socket.send_to(&packet_ser, remote_on).await;
        });
        println!("{} {} {}", self.listen_on, self.remote_on, name);
        let addr = self.listen_on.clone();
        task::block_on( async move {
            let socket = UdpSocket::bind(addr).await.unwrap();
            let mut buf = vec![0u8; 1024];
            let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
            let packet: Packet = deserialize(&buf[..n]).unwrap();
            let _r = s.send(packet);
        });
        r.recv().unwrap()

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    #[test]
    fn ideal_setup() {
        let mut requestor = CopernicaRequestor::new("127.0.0.1:8091".into(), "127.0.0.1:8090".into());
        let packet1 = requestor.request("hello1".into());
        let packet2 = requestor.request("hello2".into());
        assert_eq!(packet1, response("hello1".to_string(), "hello1".to_string().as_bytes().to_vec()));
    }
}
