use {
    copernica_lib::{Router, Config},
    packets::{mk_response, Data, ChunkBytes, Packet},
    std::{
        str::FromStr,
        str,
        env,
        fs,
        path::PathBuf,
        io::Write,
        thread::{spawn},
        collections::HashMap,
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
fn generate_random_dir_name() -> PathBuf {
    use std::iter;
    use rand::{Rng, thread_rng};
    use rand::distributions::Alphanumeric;

    let mut rng = thread_rng();
    let unique_dir: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(7)
            .collect();

    let mut dir = env::temp_dir();
    dir.push("copernica");
    dir.push(unique_dir);
    fs::create_dir_all(dir.clone());
    dir
}
fn populate_tmp_dir_dispersed_gt_mtu(node_count: usize) -> Vec<String> {
    let mut tmp_dirs: Vec<PathBuf> = Vec::with_capacity(node_count);
    const DATA_COUNT: usize = 1;
    for n in 0..node_count {
        tmp_dirs.push(generate_random_dir_name());
    }
    println!("{:?}", tmp_dirs);
    let mut all_packets: HashMap<String, Packet> = HashMap::new();
    for n in 0..DATA_COUNT{
        let name = format!("hello{}", n.clone());
        let value: ChunkBytes = vec![n.clone() as u8; 1024 * DATA_COUNT];
        let packets = mk_response(name, value);
        for (name, packet) in packets {
            all_packets.insert(name, packet);
        }
    }
    let mut current_tmp_dir = 0;
    for (name, packet) in &all_packets {
        let file = tmp_dirs[current_tmp_dir].join(name.clone());
        let mut f = fs::File::create(file).unwrap();
        let packet_ser = bincode::serialize(&packet).unwrap();
        f.write_all(&packet_ser).unwrap();
        f.sync_all().unwrap();
        if current_tmp_dir == node_count -1 {
            current_tmp_dir = 0;
        } else {
            current_tmp_dir += 1;
        }
    }
    tmp_dirs.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<String>>()
}
fn populate_tmp_dir(name: String, data: u8, size: usize) -> String {
    let value: ChunkBytes = vec![data; size];
    let packets = mk_response(name.clone().to_string(), value);
    let dir = generate_random_dir_name();
    for (name, packet) in packets {
        let dir = dir.join(name.clone());
        let mut f = fs::File::create(dir).unwrap();
        let packet_ser = bincode::serialize(&packet).unwrap();
        f.write_all(&packet_ser).unwrap();
        f.sync_all().unwrap();
    }
    dir.clone().to_string_lossy().to_string()
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
    fn small_world_graph_lt_mtu() {
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
            ], 3000);
        let mut expected: HashMap<String, Option<Packet>> = HashMap::new();
            let value0: Data = Data::Content{bytes: vec![0; 1024]};
            expected.insert("hello0".to_string(), Some(response("hello0".to_string(),value0)));
            let value1: Data = Data::Content{bytes: vec![1; 1024]};
            expected.insert("hello1".to_string(), Some(response("hello1".to_string(),value1)));
            let value2: Data = Data::Content{bytes: vec![2; 1024]};
            expected.insert("hello2".to_string(), Some(response("hello2".to_string(),value2)));
            let value3: Data = Data::Content{bytes: vec![3; 1024]};
            expected.insert("hello3".to_string(), Some(response("hello3".to_string(),value3)));
            let value4: Data = Data::Content{bytes: vec![4; 1024]};
            expected.insert("hello4".to_string(), Some(response("hello4".to_string(),value4)));
            let value5: Data = Data::Content{bytes: vec![5; 1024]};
            expected.insert("hello5".to_string(), Some(response("hello5".to_string(),value5)));
            let value6: Data = Data::Content{bytes: vec![6; 1024]};
            expected.insert("hello6".to_string(), Some(response("hello6".to_string(),value6)));
            let value7: Data = Data::Content{bytes: vec![7; 1024]};
            expected.insert("hello7".to_string(), Some(response("hello7".to_string(),value7)));
            let value8: Data = Data::Content{bytes: vec![8; 1024]};
            expected.insert("hello8".to_string(), Some(response("hello8".to_string(),value8)));
            let value9: Data = Data::Content{bytes: vec![9; 1024]};
            expected.insert("hello9".to_string(), Some(response("hello9".to_string(),value9)));
            let value10: Data = Data::Content{bytes: vec![10; 1024]};
            expected.insert("hello10".to_string(), Some(response("hello10".to_string(),value10)));
            let value11: Data = Data::Content{bytes: vec![11; 1024]};
            expected.insert("hello11".to_string(), Some(response("hello11".to_string(),value11)));
        assert_eq!(actual, expected);
    }
/*
    #[test]
    fn small_world_graph_gt_mtu() {
        // https://en.wikipedia.org/wiki/File:Small-world-network-example.png
        // node0 is 12 o'clock, node1 is 1 o'clock, etc.
        //logger::setup_logging(3, None).unwrap();
        let tmp_dirs = populate_tmp_dir_dispersed_gt_mtu(12);
        let network: Vec<Config> = vec![
            Config { listen_addr: "127.0.0.1:50020".parse().unwrap(), content_store_size: 150,
                     peers: Some(vec!["127.0.0.1:50021".into(),
                                      "127.0.0.1:50022".into(),
                                      "127.0.0.1:50023".into(),
                                      "127.0.0.1:50025".into(),
                                      "127.0.0.1:50027".into(),
                                      "127.0.0.1:50029".into(),
                                      "127.0.0.1:50030".into(),
                                      "127.0.0.1:50031".into()]),
                     data_dir: tmp_dirs[0].clone(),
            },
            Config { listen_addr: "127.0.0.1:50021".parse().unwrap(), content_store_size: 150,
                     peers: Some(vec!["127.0.0.1:50020".into(),
                                      "127.0.0.1:50022".into()]),
                     data_dir: tmp_dirs[1].clone(),
            },
            Config { listen_addr: "127.0.0.1:50022".parse().unwrap(), content_store_size: 150,
                     peers: Some(vec!["127.0.0.1:50020".into(),
                                      "127.0.0.1:50021".into(),
                                      "127.0.0.1:50023".into(),
                                      "127.0.0.1:50024".into()]),
                     data_dir: tmp_dirs[2].clone(),
            },
            Config { listen_addr: "127.0.0.1:50023".parse().unwrap(), content_store_size: 150,
                     peers: Some(vec!["127.0.0.1:50020".into(),
                                      "127.0.0.1:50022".into(),
                                      "127.0.0.1:50024".into(),
                                      "127.0.0.1:50027".into()]),
                     data_dir: tmp_dirs[3].clone(),
            },
            Config { listen_addr: "127.0.0.1:50024".parse().unwrap(), content_store_size: 150,
                     peers: Some(vec!["127.0.0.1:50022".into(),
                                      "127.0.0.1:50023".into(),
                                      "127.0.0.1:50025".into()]),
                     data_dir: tmp_dirs[4].clone(),
            },
            Config { listen_addr: "127.0.0.1:50025".parse().unwrap(), content_store_size: 150,
                     peers: Some(vec!["127.0.0.1:50020".into(),
                                      "127.0.0.1:50024".into(),
                                      "127.0.0.1:50026".into()]),
                     data_dir: tmp_dirs[5].clone(),
            },
            Config { listen_addr: "127.0.0.1:50026".parse().unwrap(), content_store_size: 150,
                     peers: Some(vec!["127.0.0.1:50025".into(),
                                      "127.0.0.1:50027".into(),
                                      "127.0.0.1:50028".into()]),
                     data_dir: tmp_dirs[6].clone(),
            },
            Config { listen_addr: "127.0.0.1:50027".parse().unwrap(), content_store_size: 150,
                     peers: Some(vec!["127.0.0.1:50020".into(),
                                      "127.0.0.1:50023".into(),
                                      "127.0.0.1:50026".into(),
                                      "127.0.0.1:50028".into(),
                                      "127.0.0.1:50029".into(),
                                      "127.0.0.1:50030".into()]),
                     data_dir: tmp_dirs[7].clone(),
            },
            Config { listen_addr: "127.0.0.1:50028".parse().unwrap(), content_store_size: 150,
                     peers: Some(vec!["127.0.0.1:50026".into(),
                                      "127.0.0.1:50027".into(),
                                      "127.0.0.1:50029".into()]),
                     data_dir: tmp_dirs[8].clone(),
            },
            Config { listen_addr: "127.0.0.1:50029".parse().unwrap(), content_store_size: 150,
                     peers: Some(vec!["127.0.0.1:50027".into(),
                                      "127.0.0.1:50028".into(),
                                      "127.0.0.1:50030".into(),
                                      "127.0.0.1:50020".into()]),
                     data_dir: tmp_dirs[9].clone(),
            },
            Config { listen_addr: "127.0.0.1:50030".parse().unwrap(), content_store_size: 150,
                     peers: Some(vec!["127.0.0.1:50027".into(),
                                      "127.0.0.1:50029".into(),
                                      "127.0.0.1:50031".into(),
                                      "127.0.0.1:50020".into()]),
                     data_dir: tmp_dirs[10].clone(),
            },
            Config { listen_addr: "127.0.0.1:50031".parse().unwrap(), content_store_size: 150,
                     peers: Some(vec!["127.0.0.1:50030".into(),
                                      "127.0.0.1:50020".into()]),
                     data_dir: tmp_dirs[11].clone(),
        }];
        setup_network(network);
        std::thread::sleep(std::time::Duration::from_secs(5));
        let mut cc = CopernicaRequestor::new("127.0.0.1:50024".into());
        let expected: ChunkBytes = vec![1; 1024 * 1];
        let actual = cc.resolve("hello1".to_string(), 6000);
        assert_eq!(actual, expected);
/*        for n in 0..11 {
            let expected: ChunkBytes = vec![n; 1024 * 12];
            let actual = cc.resolve(format!("hello{}", n), 6000);
            assert_eq!(actual, expected);
        }
*/
    }
*/
    #[test]
    fn single_fetch() {
            //populate_tmp_dir_dispersed_gt_mtu(3);
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
            let actual = cc.request(vec![ "hello3".to_string(), "hello0".to_string()], 300);
            let mut expected: HashMap<String, Option<Packet>> = HashMap::new();
            let value0: Data = Data::Content{bytes: vec![0; 1024]};
            let value3: Data = Data::Content{bytes: vec![3; 1024]};
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
        let value0: Data = Data::Manifest{chunk_count:1};
        expected.insert("hello".to_string(), Some(response("hello".to_string(),value0)));
        let value1: Data = Data::Content{bytes:vec![0; 1024]};
        expected.insert("hello-0".to_string(), Some(response("hello-0".to_string(),value1)));
        let value2: Data = Data::Content{bytes:vec![0; 1]};
        expected.insert("hello-1".to_string(), Some(response("hello-1".to_string(),value2)));
        assert_eq!(actual, expected);

    }

    #[test]
    fn resolve_gt_mtu() {
        //logger::setup_logging(3, None).unwrap();
        let network: Vec<Config> = vec![
            Config {
                listen_addr: "127.0.0.1:50106".parse().unwrap(),
                content_store_size: 5000,
                peers: None,
                data_dir: populate_tmp_dir("hello".to_string(), 0, 1025),
            },
        ];
        setup_network(network);
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let mut cc = CopernicaRequestor::new("127.0.0.1:50106".into());
        let actual = cc.resolve("hello".to_string(), 2000);
        let mut expected: ChunkBytes = vec![0; 1025];
        assert_eq!(actual, expected);
    }

    #[test]
    fn resolve_lt_mtu() {
        //logger::setup_logging(3, None).unwrap();
        let network: Vec<Config> = vec![
            Config {
                listen_addr: "127.0.0.1:50107".parse().unwrap(),
                content_store_size: 50,
                peers: None,
                data_dir: populate_tmp_dir("hello".to_string(), 0, 1023),
            },
        ];
        setup_network(network);
        let mut cc = CopernicaRequestor::new("127.0.0.1:50107".into());
        let actual = cc.resolve("hello".to_string(), 500);
        let mut expected: ChunkBytes = vec![0; 1023];
        assert_eq!(actual, expected);
    }
}
