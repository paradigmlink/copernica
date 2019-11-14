use {
    client::{CopernicaRequestor},
    crossbeam_channel::{
        Receiver,
        unbounded,
    },
    packets::{Packet, response, request},
    logger,
    router::{Router, Config},
    std::{
        str::FromStr,
        str,
        thread::{spawn, sleep, JoinHandle},
        time,
        io,
        collections::HashMap,
    },
    log::{trace},
};


#[derive(Debug, PartialEq)]
struct Data {
    name: String,
    data: String,
}

impl FromStr for Data {
    type Err = std::net::AddrParseError;

    fn from_str(socket_pair: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = socket_pair.split('|').collect();
        let name: String = v[0].to_string();
        let data: String = v[1].to_string();
        Ok( Data { name: name, data: data })
    }

}

fn router(listen_addr: String, peers: Option<Vec<String>>, data: Option<Vec<Data>>) -> JoinHandle<()> {
    let config = Config {
        listen_addr: listen_addr,
        content_store_size: 50,
        peers: peers,
    };
    let router = spawn( move || {
        let mut r = Router::new_with_config(config);
        match data {
            Some(data) => {
                for item in data {
                    r.insert_into_cs(response(item.name, item.data.as_bytes().to_vec()));
                }
            },
            None => {},
        }
        r.run()
    });
    router
}

fn simple_network() {
    let mut threads: Vec<JoinHandle<()>> = vec![];
    let node0 = "127.0.0.1:50100".into();
    let peer0 = vec!["127.0.0.1:50101".into()];
    let data0 = vec![Data::from_str("hello0|world").unwrap()];
    threads.push(router(node0, Some(peer0), Some(data0)));
    let node1 = "127.0.0.1:50101".into();
    let peer1 = vec!["127.0.0.1:50102".into()];
    let data1 = vec![Data::from_str("hello1|world").unwrap()];
    threads.push(router(node1, Some(peer1), Some(data1)));
    let node2 = "127.0.0.1:50102".into();
    let peer2 = vec!["127.0.0.1:50103".into()];
    let data2 = vec![Data::from_str("hello2|world").unwrap()];
    threads.push(router(node2, Some(peer2), Some(data2)));
    let node3 = "127.0.0.1:50103".into();
    let data3 = vec![Data::from_str("hello3|world").unwrap()];
    threads.push(router(node3, None, Some(data3)));
}

fn small_world_graph()  {
    // https://en.wikipedia.org/wiki/File:Small-world-network-example.png
    // node0 is 12 o'clock, node1 is 1 o'clock, etc.
    let mut threads: Vec<JoinHandle<()>> = vec![];
    let node0 =      "127.0.0.1:50000".into();
    let peer0 = vec!["127.0.0.1:50001".into(),
                     "127.0.0.1:50002".into(),
                     "127.0.0.1:50003".into(),
                     "127.0.0.1:50005".into(),
                     "127.0.0.1:50007".into(),
                     "127.0.0.1:50009".into(),
                     "127.0.0.1:50010".into(),
                     "127.0.0.1:50011".into()];
    let data0 = vec![Data::from_str("hello0|world").unwrap()];
    threads.push(router(node0, Some(peer0), Some(data0)));

    let node1 =      "127.0.0.1:50001".into();
    let peer1 = vec!["127.0.0.1:50000".into(),
                     "127.0.0.1:50002".into()];
    let data1 = vec![Data::from_str("hello1|world").unwrap()];
    threads.push(router(node1, Some(peer1), Some(data1)));

    let node2 =      "127.0.0.1:50002".into();
    let peer2 = vec!["127.0.0.1:50000".into(),
                     "127.0.0.1:50001".into(),
                     "127.0.0.1:50003".into(),
                     "127.0.0.1:50004".into()];
    let data2 = vec![Data::from_str("hello2|world").unwrap()];
    threads.push(router(node2, Some(peer2), Some(data2)));

    let node3 =      "127.0.0.1:50003".into();
    let peer3 = vec!["127.0.0.1:50000".into(),
                     "127.0.0.1:50002".into(),
                     "127.0.0.1:50004".into(),
                     "127.0.0.1:50007".into()];
    let data3 = vec![Data::from_str("hello3|world").unwrap()];
    threads.push(router(node3, Some(peer3), Some(data3)));

    let node4 =      "127.0.0.1:50004".into();
    let peer4 = vec!["127.0.0.1:50002".into(),
                     "127.0.0.1:50003".into(),
                     "127.0.0.1:50005".into()];
    let data4 = vec![Data::from_str("hello4|world").unwrap()];
    threads.push(router(node4, Some(peer4), Some(data4)));

    let node5 =      "127.0.0.1:50005".into();
    let peer5 = vec!["127.0.0.1:50000".into(),
                     "127.0.0.1:50004".into(),
                     "127.0.0.1:50006".into()];
    let data5 = vec![Data::from_str("hello5|world").unwrap()];
    threads.push(router(node5, Some(peer5), Some(data5)));

    let node6 =      "127.0.0.1:50006".into();
    let peer6 = vec!["127.0.0.1:50005".into(),
                     "127.0.0.1:50007".into(),
                     "127.0.0.1:50008".into()];
    let data6 = vec![Data::from_str("hello6|world").unwrap()];
    threads.push(router(node6, Some(peer6), Some(data6)));

    let node7 =      "127.0.0.1:50007".into();
    let peer7 = vec!["127.0.0.1:50000".into(),
                     "127.0.0.1:50003".into(),
                     "127.0.0.1:50006".into(),
                     "127.0.0.1:50008".into(),
                     "127.0.0.1:50009".into(),
                     "127.0.0.1:50010".into()];
    let data7 = vec![Data::from_str("hello7|world").unwrap()];
    threads.push(router(node7, Some(peer7), Some(data7)));

    let node8 =      "127.0.0.1:50008".into();
    let peer8 = vec!["127.0.0.1:50006".into(),
                     "127.0.0.1:50007".into(),
                     "127.0.0.1:50009".into()];
    let data8 = vec![Data::from_str("hello8|world").unwrap()];
    threads.push(router(node8, Some(peer8), Some(data8)));

    let node9 =      "127.0.0.1:50009".into();
    let peer9 = vec!["127.0.0.1:50007".into(),
                     "127.0.0.1:50008".into(),
                     "127.0.0.1:50010".into(),
                     "127.0.0.1:50000".into()];
    let data9 = vec![Data::from_str("hello9|world").unwrap()];
    threads.push(router(node9, Some(peer9), Some(data9)));

    let node10 =      "127.0.0.1:50010".into();
    let peer10 = vec!["127.0.0.1:50007".into(),
                      "127.0.0.1:50009".into(),
                      "127.0.0.1:50011".into(),
                      "127.0.0.1:50000".into()];
    let data10 = vec![Data::from_str("hello10|world").unwrap()];
    threads.push(router(node10, Some(peer10), Some(data10)));

    let node11 =      "127.0.0.1:50011".into();
    let peer11 = vec!["127.0.0.1:50010".into(),
                      "127.0.0.1:50000".into()];
    let data11 = vec![Data::from_str("hello11|world").unwrap()];
    threads.push(router(node11, Some(peer11), Some(data11)));
}

fn main() {
    //logger::setup_logging(3, None).unwrap();
    small_world_graph();
    trace!("finished small world");
}

#[cfg(test)]
mod network_regressions {
    use super::*;

    #[test]
    fn a_simple_single_fetch() {
        logger::setup_logging(3, None).unwrap();
        simple_network();
        let mut cc = CopernicaRequestor::new("127.0.0.1:50100".into());
        let actual = cc.request(vec![ "hello3".to_string() ]);
        let mut expected: HashMap<String, Option<Packet>> = HashMap::new();
        expected.insert("hello3".to_string(), Some(response("hello3".to_string(),"world".to_string().as_bytes().to_vec())));
        assert_eq!(actual, expected);
    }
/*
    #[test]
    fn timeout() {
        logger::setup_logging(3, None).unwrap();
        simple_network();
        let mut cc = CopernicaRequestor::new("127.0.0.1:50100".into());
        let actual = cc.request(vec![ "hello4".to_string() ]);
        let mut expected: HashMap<String, Option<Packet>> = HashMap::new();
        expected.insert("hello2".to_string(), None);
        assert_eq!(actual, expected);
    }
*/
    #[test]
    fn a_small_world_graph() {
        //logger::setup_logging(3, None).unwrap();
        small_world_graph();

        let mut cc = CopernicaRequestor::new("127.0.0.1:50004".into());
        let actual = cc.request(vec![
            "hello0".to_string(),
            "hello1".to_string(),
            "hello2".to_string(),
            "hello3".to_string(),
            "hello4".to_string(),
            "hello5".to_string(),
            "hello6".to_string(),
            "hello7".to_string(),
            "hello8".to_string(),
            "hello9".to_string(),
            "hello10".to_string(),
            "hello11".to_string(),
            ]);
        let mut expected: HashMap<String, Option<Packet>> = HashMap::new();
        expected.insert("hello0".to_string(), Some(response("hello0".to_string(),"world".to_string().as_bytes().to_vec())));
        expected.insert("hello1".to_string(), Some(response("hello1".to_string(),"world".to_string().as_bytes().to_vec())));
        expected.insert("hello2".to_string(), Some(response("hello2".to_string(),"world".to_string().as_bytes().to_vec())));
        expected.insert("hello3".to_string(), Some(response("hello3".to_string(),"world".to_string().as_bytes().to_vec())));
        expected.insert("hello4".to_string(), Some(response("hello4".to_string(),"world".to_string().as_bytes().to_vec())));
        expected.insert("hello5".to_string(), Some(response("hello5".to_string(),"world".to_string().as_bytes().to_vec())));
        expected.insert("hello6".to_string(), Some(response("hello6".to_string(),"world".to_string().as_bytes().to_vec())));
        expected.insert("hello7".to_string(), Some(response("hello7".to_string(),"world".to_string().as_bytes().to_vec())));
        expected.insert("hello8".to_string(), Some(response("hello8".to_string(),"world".to_string().as_bytes().to_vec())));
        expected.insert("hello9".to_string(), Some(response("hello9".to_string(),"world".to_string().as_bytes().to_vec())));
        expected.insert("hello10".to_string(), Some(response("hello10".to_string(),"world".to_string().as_bytes().to_vec())));
        expected.insert("hello11".to_string(), Some(response("hello11".to_string(),"world".to_string().as_bytes().to_vec())));
        assert_eq!(actual, expected);
    }

}
