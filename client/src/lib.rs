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

#[derive( Clone)]
pub struct CopernicaClient {
    listen_addr: SocketAddrV4,
    remote_addr: SocketAddrV4,
    inbound_sender: Sender<Packet>,
    inbound_receiver: Receiver<Packet>,
}

impl CopernicaClient {
    pub fn new(listen_addr: String, remote_addr: String) -> (CopernicaClient, Receiver<Packet>) {
        let (inbound_sender, inbound_receiver) = unbounded();
        (CopernicaClient {
            listen_addr: listen_addr.parse().unwrap(),
            remote_addr: remote_addr.parse().unwrap(),
            inbound_sender: inbound_sender,
            inbound_receiver: inbound_receiver.clone(),
        }, inbound_receiver.clone())
    }

    pub async fn inbound(&self) {
        let socket = UdpSocket::bind(self.listen_addr).await.unwrap();
        loop {
            let mut buf = vec![0u8; 1024];
            let (n, _peer) = socket.recv_from(&mut buf).await.unwrap();
            let packet: Packet = deserialize(&buf[..n]).unwrap();
            let _r = self.inbound_sender.send(packet);
        }
    }
    pub fn outbound(&self, packet: Packet) {
        task::block_on(async {
            let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let packet_ser = serialize(&packet).unwrap();
            let _r = socket.send_to(&packet_ser, self.remote_addr.clone()).await;
        })
    }
}

#[cfg(test)]
mod client {
    use {
        super::*,
        futures::executor::ThreadPool,
    };

    #[test]
    fn basic_setup() {
        let mut executor = ThreadPool::new().unwrap();
        let (cc, inbound) = CopernicaClient::new("127.0.0.1:8091".into(), "127.0.0.1:8091".into());
        let ccc = cc.clone();
        std::thread::spawn(move || { executor.run(ccc.inbound()) });
        std::thread::sleep(std::time::Duration::from_millis(2));
        cc.outbound(request("hello1".into()));
        let packet = inbound.recv().unwrap();
        assert_eq!(packet, request("hello1".to_string()));
    }
}
