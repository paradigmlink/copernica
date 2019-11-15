use {
    client::{CopernicaRequestor},
    packets::{Packet, response},
    router::{Router, Config, NamedData},
    std::{
        str::FromStr,
        str,
        thread::{spawn},
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

fn router(config: Config) {
    spawn( move || {
        let mut r = Router::new_with_config(config);
        r.run()
    });
}

fn setup_network(network: Vec<Config>) {
    for node in network {
        router(node);
    }
}

fn main() {
    //logger::setup_logging(3, None).unwrap();
    //small_world_graph();
    trace!("finished small world");
}

#[cfg(test)]
mod network_regressions {
    use super::*;

    #[test]
    fn small_world_graph() {
        // https://en.wikipedia.org/wiki/File:Small-world-network-example.png
        // node0 is 12 o'clock, node1 is 1 o'clock, etc.
        //logger::setup_logging(3, None).unwrap();
        let network: Vec<Config> = vec![
            Config { listen_addr: "127.0.0.1:50000".into(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50001".into(),
                                      "127.0.0.1:50002".into(),
                                      "127.0.0.1:50003".into(),
                                      "127.0.0.1:50005".into(),
                                      "127.0.0.1:50007".into(),
                                      "127.0.0.1:50009".into(),
                                      "127.0.0.1:50010".into(),
                                      "127.0.0.1:50011".into()]),
                     data: Some(vec![NamedData{ name: "hello0".into(), data: "world".into()}]),
            },
            Config { listen_addr: "127.0.0.1:50001".into(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50000".into(),
                                      "127.0.0.1:50002".into()]),
                     data: Some(vec![NamedData{ name: "hello1".into(), data: "world".into()}]),
            },
            Config { listen_addr: "127.0.0.1:50002".into(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50000".into(),
                                      "127.0.0.1:50001".into(),
                                      "127.0.0.1:50003".into(),
                                      "127.0.0.1:50004".into()]),
                     data: Some(vec![NamedData{ name: "hello2".into(), data: "world".into()}]),
            },
            Config { listen_addr: "127.0.0.1:50003".into(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50000".into(),
                                      "127.0.0.1:50002".into(),
                                      "127.0.0.1:50004".into(),
                                      "127.0.0.1:50007".into()]),
                     data: Some(vec![NamedData{ name: "hello3".into(), data: "world".into()}]),
            },
            Config { listen_addr: "127.0.0.1:50004".into(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50002".into(),
                                      "127.0.0.1:50003".into(),
                                      "127.0.0.1:50005".into()]),
                     data: Some(vec![NamedData{ name: "hello4".into(), data: "world".into()}]),
            },
            Config { listen_addr: "127.0.0.1:50005".into(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50000".into(),
                                      "127.0.0.1:50004".into(),
                                      "127.0.0.1:50006".into()]),
                     data: Some(vec![NamedData{ name: "hello5".into(), data: "world".into()}]),
            },
            Config { listen_addr: "127.0.0.1:50006".into(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50005".into(),
                                      "127.0.0.1:50007".into(),
                                      "127.0.0.1:50008".into()]),
                     data: Some(vec![NamedData{ name: "hello6".into(), data: "world".into()}]),
            },
            Config { listen_addr: "127.0.0.1:50007".into(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50000".into(),
                                      "127.0.0.1:50003".into(),
                                      "127.0.0.1:50006".into(),
                                      "127.0.0.1:50008".into(),
                                      "127.0.0.1:50009".into(),
                                      "127.0.0.1:50010".into()]),
                     data: Some(vec![NamedData{ name: "hello7".into(), data: "world".into()}]),
            },
            Config { listen_addr: "127.0.0.1:50008".into(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50006".into(),
                                      "127.0.0.1:50007".into(),
                                      "127.0.0.1:50009".into()]),
                     data: Some(vec![NamedData{ name: "hello8".into(), data: "world".into()}]),
            },
            Config { listen_addr: "127.0.0.1:50009".into(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50007".into(),
                                      "127.0.0.1:50008".into(),
                                      "127.0.0.1:50010".into(),
                                      "127.0.0.1:50000".into()]),
                     data: Some(vec![NamedData{ name: "hello9".into(), data: "world".into()}]),
            },
            Config { listen_addr: "127.0.0.1:50010".into(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50007".into(),
                                      "127.0.0.1:50009".into(),
                                      "127.0.0.1:50011".into(),
                                      "127.0.0.1:50000".into()]),
                     data: Some(vec![NamedData{ name: "hello10".into(), data: "world".into()}]),
            },
            Config { listen_addr: "127.0.0.1:50011".into(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50010".into(),
                                      "127.0.0.1:50000".into()]),
                     data: Some(vec![NamedData{ name: "hello11".into(), data: "world".into()}]),
        }];
        setup_network(network);
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
            ], 1500);
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
    #[test]
    fn a_simple_single_fetch() {
//            logger::setup_logging(3, None).unwrap();
            let mut network: Vec<Config> = vec![];
            network.push(Config {
                listen_addr: "127.0.0.1:50100".into(),
                content_store_size: 50,
                peers: Some(vec!["127.0.0.1:50101".into()]),
                data: Some(vec![NamedData{ name: "hello0".into(), data: "world".into()}]),
            });
            network.push(Config {
                listen_addr: "127.0.0.1:50101".into(),
                content_store_size: 50,
                peers: Some(vec!["127.0.0.1:50102".into()]),
                data: Some(vec![NamedData{ name: "hello1".into(), data: "world".into()}]),
            });
            network.push(Config {
                listen_addr: "127.0.0.1:50102".into(),
                content_store_size: 50,
                peers: Some(vec!["127.0.0.1:50103".into()]),
                data: Some(vec![NamedData{ name: "hello2".into(), data: "world".into()}]),
            });
            network.push(Config {
                listen_addr: "127.0.0.1:50103".into(),
                content_store_size: 50,
                peers: None,
                data: Some(vec![NamedData{ name: "hello3".into(), data: "world".into()}]),
            });
            setup_network(network);
            let mut cc = CopernicaRequestor::new("127.0.0.1:50100".into());
            let actual = cc.request(vec![ "hello3".to_string()], 200);
            let mut expected: HashMap<String, Option<Packet>> = HashMap::new();
            expected.insert("hello3".to_string(), Some(response("hello3".to_string(),"world".to_string().as_bytes().to_vec())));
            assert_eq!(actual, expected);
        }

    #[test]
    fn timeout() {
        //logger::setup_logging(3, None).unwrap();
        let network: Vec<Config> = vec![
            Config {
                listen_addr: "127.0.0.1:50104".into(),
                content_store_size: 50,
                peers: None,
                data: Some(vec![NamedData{ name: "hello0".into(), data: "world".into()}]),
            }
        ];
        setup_network(network);
        let mut cc = CopernicaRequestor::new("127.0.0.1:50104".into());
        let actual = cc.request(vec![ "hello1".to_string()], 50);
        let mut expected: HashMap<String, Option<Packet>> = HashMap::new();
        expected.insert("hello1".to_string(), None);
        assert_eq!(actual, expected);
    }
}
