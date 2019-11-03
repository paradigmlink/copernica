// @implement: listen_for_requests
use {
    packets::{Packet, Sdri, Data, generate_sdr_index, response, request},
    bincode::{serialize, deserialize},
    std::{
        net::{SocketAddrV4},
        collections::HashMap,
        sync::{Arc, Mutex},
    },
    crossbeam_channel::{Sender, Receiver, unbounded, Select},
    async_std::{
        net::UdpSocket,
        task,
    },
    futures::executor::ThreadPool,
};

#[derive( Clone)]
pub struct CopernicaClient {
    listen_addr: SocketAddrV4,
    remote_addr: SocketAddrV4,
    inbound_response_sender: Sender<Packet>,
    inbound_response_receiver: Receiver<Packet>,
    inbound_request_sender: Sender<String>,
    inbound_request_receiver: Receiver<String>,
    response_cache: Arc<Mutex<HashMap<Sdri, Packet>>>,
    listen_for: Arc<Mutex<HashMap<Sdri, String>>>,
}

impl CopernicaClient {
    pub fn new(listen_addr: String, remote_addr: String) -> CopernicaClient {
        let (inbound_sender, inbound_receiver) = unbounded();
        let (inbound_request_sender, inbound_request_receiver) = unbounded();
        let listen_addr: SocketAddrV4 = listen_addr.parse().unwrap();
        CopernicaClient {
            listen_addr: listen_addr,
            remote_addr: remote_addr.parse().unwrap(),
            inbound_response_sender: inbound_sender,
            inbound_response_receiver: inbound_receiver.clone(),
            inbound_request_sender: inbound_request_sender,
            inbound_request_receiver: inbound_request_receiver.clone(),
            response_cache: Arc::new(Mutex::new(HashMap::new())),
            listen_for: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(&mut self) {
        let socket = UdpSocket::bind(self.listen_addr).await.unwrap();
        loop {
            let mut buf = vec![0u8; 1024];
            let (n, _peer) = socket.recv_from(&mut buf).await.unwrap();
            let packet: Packet = deserialize(&buf[..n]).unwrap();
            match packet.clone() {
                Packet::Request { sdri } => {
                    if let Some(p) = self.response_cache.lock().unwrap().get(&sdri) {
                        self.outbound(p.clone());
                    }
                    if let Some(n) = self.listen_for.lock().unwrap().get(&sdri) {
                        self.inbound_request_sender.send(n.to_string());
                    }
                },
                Packet::Response { sdri, data } => {
                    if self.listen_for.lock().unwrap().contains_key(&sdri) {
                        self.response_cache.lock().unwrap().insert(sdri, packet.clone());
                        self.inbound_response_sender.send(packet).unwrap();
                    }
                },
            }
        }
    }
    fn outbound(&self, packet: Packet) {
        task::block_on(async {
            let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let packet_ser = serialize(&packet).unwrap();
            let _r = socket.send_to(&packet_ser, self.remote_addr.clone()).await;
        });
    }
    pub fn listen_for_requests(&mut self, names: Vec<String>) -> Receiver<String> {
        for name in names {
            self.listen_for.lock().unwrap().insert(generate_sdr_index(name.clone()), name);
        }
        self.inbound_request_receiver.clone()
    }
    pub fn respond(&self, packet: Packet) {
        self.outbound(packet);
    }
    pub fn request_one(&mut self, name: String) -> Option<Packet> {
        let index = generate_sdr_index(name.clone());
        self.listen_for.lock().unwrap().insert(index.clone(), name.clone());
        if let Some(p) = self.response_cache.lock().unwrap().get(&index) {
            return Some(p.clone())
        }
        let packet = request(name);
        let mut sel = Select::new();
        let oper1 = sel.recv(&self.inbound_response_receiver);
        self.outbound(packet);
        match sel.ready_timeout(std::time::Duration::from_millis(500)) {
            Err(_) => { println!("should not have timed out"); return None},
            Ok(i) if i == oper1 => {
                    let packet: Packet = self.inbound_response_receiver.try_recv().unwrap();
                    match packet.clone() {
                        Packet::Request { sdri } => unreachable!(),
                        Packet::Response { sdri, data } => {
                            if sdri == index {
                                return Some(packet)
                            } else {
                                return None
                            }
                        },
                    }
                },
            Ok(_) => unreachable!(),
        }
    }
    pub fn request_many(&mut self, mut names: Vec<String>) -> Vec<(String, Option<Packet>)> {
        names.iter_mut().map(| name | (name.clone().to_string(), self.request_one(name.to_string()))).collect::<Vec<(String, Option<Packet>)>>()
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
