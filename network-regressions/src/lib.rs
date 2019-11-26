use {
    copernica_lib::{Router, Config},
    packets::{mk_response, Data},
    std::{
        str::FromStr,
        str,
        env,
        fs,
        io::Write,
        thread::{spawn},
    },
    bincode,
    log::{trace},
};

#[allow(dead_code)]
fn router(config: Config) {
    spawn( move || {
        let mut r = Router::new_with_config(config);
        r.run()
    });
}
#[allow(dead_code)]
fn setup_network(network: Vec<Config>) {
    for node in network {
        router(node);
    }
}

fn populate_tmp_dir(name: String, data: u8, size: usize) -> String {
    use std::iter;
    use rand::{Rng, thread_rng};
    use rand::distributions::Alphanumeric;

    let mut rng = thread_rng();
    let unique_dir: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(7)
            .collect();
    let value: Data = vec![data; size];
    let packets = mk_response(name.clone().to_string(), value);
    let mut dir = env::temp_dir();
    dir.push("copernica");
    dir.push(unique_dir);
    let out = dir.clone().to_string_lossy().to_string();
    fs::create_dir_all(dir.clone());
    for (name, packet) in packets {
        let dir = dir.join(name.clone());
        let mut f = fs::File::create(dir).unwrap();
        let packet_ser = bincode::serialize(&packet).unwrap();
        f.write_all(&packet_ser).unwrap();
        f.sync_all().unwrap();
    }
    out
}

#[cfg(test)]
mod network_regressions {
    use super::*;

    use {
        packets::{Packet, response},
        copernica_lib::{Config, CopernicaRequestor},
        std::{
            collections::HashMap,
        },
        dirs,
    };

    #[test]
    fn small_world_graph() {
        // https://en.wikipedia.org/wiki/File:Small-world-network-example.png
        // node0 is 12 o'clock, node1 is 1 o'clock, etc.
        //logger::setup_logging(3, None).unwrap();
        let network: Vec<Config> = vec![
            Config { listen_addr: "127.0.0.1:50000".parse().unwrap(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50001".into(),
                                      "127.0.0.1:50002".into(),
                                      "127.0.0.1:50003".into(),
                                      "127.0.0.1:50005".into(),
                                      "127.0.0.1:50007".into(),
                                      "127.0.0.1:50009".into(),
                                      "127.0.0.1:50010".into(),
                                      "127.0.0.1:50011".into()]),
                     data_dir: populate_tmp_dir("hello0".to_string(), 0, 1024),
            },
            Config { listen_addr: "127.0.0.1:50001".parse().unwrap(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50000".into(),
                                      "127.0.0.1:50002".into()]),
                     data_dir: populate_tmp_dir("hello1".to_string(), 1, 1024),
            },
            Config { listen_addr: "127.0.0.1:50002".parse().unwrap(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50000".into(),
                                      "127.0.0.1:50001".into(),
                                      "127.0.0.1:50003".into(),
                                      "127.0.0.1:50004".into()]),
                     data_dir: populate_tmp_dir("hello2".to_string(), 2, 1024),
            },
            Config { listen_addr: "127.0.0.1:50003".parse().unwrap(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50000".into(),
                                      "127.0.0.1:50002".into(),
                                      "127.0.0.1:50004".into(),
                                      "127.0.0.1:50007".into()]),
                     data_dir: populate_tmp_dir("hello3".to_string(), 3, 1024),
            },
            Config { listen_addr: "127.0.0.1:50004".parse().unwrap(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50002".into(),
                                      "127.0.0.1:50003".into(),
                                      "127.0.0.1:50005".into()]),
                     data_dir: populate_tmp_dir("hello4".to_string(), 4, 1024),
            },
            Config { listen_addr: "127.0.0.1:50005".parse().unwrap(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50000".into(),
                                      "127.0.0.1:50004".into(),
                                      "127.0.0.1:50006".into()]),
                     data_dir: populate_tmp_dir("hello5".to_string(), 5, 1024),
            },
            Config { listen_addr: "127.0.0.1:50006".parse().unwrap(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50005".into(),
                                      "127.0.0.1:50007".into(),
                                      "127.0.0.1:50008".into()]),
                     data_dir: populate_tmp_dir("hello6".to_string(), 6, 1024),
            },
            Config { listen_addr: "127.0.0.1:50007".parse().unwrap(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50000".into(),
                                      "127.0.0.1:50003".into(),
                                      "127.0.0.1:50006".into(),
                                      "127.0.0.1:50008".into(),
                                      "127.0.0.1:50009".into(),
                                      "127.0.0.1:50010".into()]),
                     data_dir: populate_tmp_dir("hello7".to_string(), 7, 1024),
            },
            Config { listen_addr: "127.0.0.1:50008".parse().unwrap(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50006".into(),
                                      "127.0.0.1:50007".into(),
                                      "127.0.0.1:50009".into()]),
                     data_dir: populate_tmp_dir("hello8".to_string(), 8, 1024),
            },
            Config { listen_addr: "127.0.0.1:50009".parse().unwrap(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50007".into(),
                                      "127.0.0.1:50008".into(),
                                      "127.0.0.1:50010".into(),
                                      "127.0.0.1:50000".into()]),
                     data_dir: populate_tmp_dir("hello9".to_string(), 9, 1024),
            },
            Config { listen_addr: "127.0.0.1:50010".parse().unwrap(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50007".into(),
                                      "127.0.0.1:50009".into(),
                                      "127.0.0.1:50011".into(),
                                      "127.0.0.1:50000".into()]),
                     data_dir: populate_tmp_dir("hello10".to_string(), 10, 1024),
            },
            Config { listen_addr: "127.0.0.1:50011".parse().unwrap(), content_store_size: 50,
                     peers: Some(vec!["127.0.0.1:50010".into(),
                                      "127.0.0.1:50000".into()]),
                     data_dir: populate_tmp_dir("hello11".to_string(), 11, 1024),
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
            ], 5000);
        let mut expected: HashMap<String, Option<Packet>> = HashMap::new();
            let value0: Data = vec![0; 1024];
            expected.insert("hello0".to_string(), Some(response("hello0".to_string(),value0)));
            let value1: Data = vec![1; 1024];
            expected.insert("hello1".to_string(), Some(response("hello1".to_string(),value1)));
            let value2: Data = vec![2; 1024];
            expected.insert("hello2".to_string(), Some(response("hello2".to_string(),value2)));
            let value3: Data = vec![3; 1024];
            expected.insert("hello3".to_string(), Some(response("hello3".to_string(),value3)));
            let value4: Data = vec![4; 1024];
            expected.insert("hello4".to_string(), Some(response("hello4".to_string(),value4)));
            let value5: Data = vec![5; 1024];
            expected.insert("hello5".to_string(), Some(response("hello5".to_string(),value5)));
            let value6: Data = vec![6; 1024];
            expected.insert("hello6".to_string(), Some(response("hello6".to_string(),value6)));
            let value7: Data = vec![7; 1024];
            expected.insert("hello7".to_string(), Some(response("hello7".to_string(),value7)));
            let value8: Data = vec![8; 1024];
            expected.insert("hello8".to_string(), Some(response("hello8".to_string(),value8)));
            let value9: Data = vec![9; 1024];
            expected.insert("hello9".to_string(), Some(response("hello9".to_string(),value9)));
            let value10: Data = vec![10; 1024];
            expected.insert("hello10".to_string(), Some(response("hello10".to_string(),value10)));
            let value11: Data = vec![11; 1024];
            expected.insert("hello11".to_string(), Some(response("hello11".to_string(),value11)));
        assert_eq!(actual, expected);
    }
    #[test]
    fn single_fetch() {
            //logger::setup_logging(3, None).unwrap();
            let mut network: Vec<Config> = vec![];
            network.push(Config {
                listen_addr: "127.0.0.1:50100".parse().unwrap(),
                content_store_size: 50,
                peers: Some(vec!["127.0.0.1:50101".into()]),
                data_dir: populate_tmp_dir("hello0".to_string(), 0, 1024),
            });
            network.push(Config {
                listen_addr: "127.0.0.1:50101".parse().unwrap(),
                content_store_size: 50,
                peers: Some(vec!["127.0.0.1:50102".into()]),
                data_dir: populate_tmp_dir("hello1".to_string(), 1, 1024),
            });
            network.push(Config {
                listen_addr: "127.0.0.1:50102".parse().unwrap(),
                content_store_size: 50,
                peers: Some(vec!["127.0.0.1:50103".into()]),
                data_dir: populate_tmp_dir("hello2".to_string(), 2, 1024),
            });
            network.push(Config {
                listen_addr: "127.0.0.1:50103".parse().unwrap(),
                content_store_size: 50,
                peers: None,
                data_dir: populate_tmp_dir("hello3".to_string(), 3, 1024),
            });
            setup_network(network);
            let mut cc = CopernicaRequestor::new("127.0.0.1:50100".into());
            let actual = cc.request(vec![ "hello3".to_string(), "hello0".to_string()], 200);
            let mut expected: HashMap<String, Option<Packet>> = HashMap::new();
            let value0: Data = vec![0; 1024];
            let value3: Data = vec![3; 1024];
            expected.insert("hello3".to_string(), Some(response("hello3".to_string(),value3)));
            expected.insert("hello0".to_string(), Some(response("hello0".to_string(),value0)));
            assert_eq!(actual, expected);
        }

    #[test]
    fn timeout() {
        //logger::setup_logging(3, None).unwrap();
        let network: Vec<Config> = vec![
            Config {
                listen_addr: "127.0.0.1:50104".parse().unwrap(),
                content_store_size: 50,
                peers: None,
                data_dir: populate_tmp_dir("hello0".to_string(), 0, 1024),
            }
        ];
        setup_network(network);
        let mut cc = CopernicaRequestor::new("127.0.0.1:50104".into());
        let actual = cc.request(vec![ "hello1".to_string()], 50);
        let mut expected: HashMap<String, Option<Packet>> = HashMap::new();
        expected.insert("hello1".to_string(), None);
        assert_eq!(actual, expected);
    }

    #[test]
    fn make_chunks() {
        //logger::setup_logging(3, None).unwrap();
        let network: Vec<Config> = vec![
            Config {
                listen_addr: "127.0.0.1:50105".parse().unwrap(),
                content_store_size: 50,
                peers: None,
                data_dir: populate_tmp_dir("hello".to_string(), 0, 1025),
            },
        ];
        setup_network(network);
        let mut cc = CopernicaRequestor::new("127.0.0.1:50105".into());
        let actual = cc.request(vec![
            "hello".to_string(),
            "hello-0".to_string(),
            "hello-1".to_string(),
        ], 500);
        let mut expected: HashMap<String, Option<Packet>> = HashMap::new();
        let value0: Data = "hello\n1".as_bytes().to_vec();
        expected.insert("hello".to_string(), Some(response("hello".to_string(),value0)));
        let value1: Data = vec![0; 1024];
        expected.insert("hello-0".to_string(), Some(response("hello-0".to_string(),value1)));
        let value2: Data = vec![0; 1];
        expected.insert("hello-1".to_string(), Some(response("hello-1".to_string(),value2)));
        assert_eq!(actual, expected);

    }
}
