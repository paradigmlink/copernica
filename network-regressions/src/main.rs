#![allow(dead_code)]
use {
    libcopernica::{
        Router, CopernicaRequestor,
        Config,
        constants,
        narrow_waist::{Bytes},
        response_store::{Response, mk_response},
    },
    async_std::{ task, },
    std::{
        env,
        fs,
        path::PathBuf,
        io::Write,
        thread::{spawn},
        collections::HashMap,
    },
    bincode,
};

//const TIMEOUT: u64 = 1000;
const GT_MTU: usize = 1410;
const GT_MTU_BY_12: usize = GT_MTU * 12;
const MB0_1: usize  = 104857;
const MB0_2: usize  = 209715;
const MB0_3: usize  = 314572;
const MB0_4: usize  = 419430;
const MB0_5: usize  = 524288;
const MB0_6: usize  = 629145;
const MB0_7: usize  = 734003;
const MB0_8: usize  = 838860;
const MB1: usize    = 1048576;
const MB5: usize    = 5242880;
const MB10: usize   = 10485760;
const MB20: usize   = 20971520;
const MB50: usize   = 52428800;
const MB100: usize  = 104857600;
const MB500: usize  = 524288000;
const MB1000: usize = 1048576000;

#[allow(dead_code)]
fn router(config: Config) {
    let mut router = Router::new_with_config(config);
    spawn( move || {
        task::block_on(async {
            router.run().await
        });
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
    fs::create_dir_all(dir.clone()).unwrap();
    dir
}
fn populate_tmp_dir_dispersed_gt_mtu(node_count: usize, data_size: usize) -> Vec<String> {
    let mut tmp_dirs: Vec<PathBuf> = Vec::with_capacity(node_count);
    for _ in 0..node_count {
        tmp_dirs.push(generate_random_dir_name());
    }
    let mut responses: HashMap<String, Response> = HashMap::new();
    for n in 0..node_count {
        let name = format!("hello{}", n.clone());
        let value: Bytes = vec![n.clone() as u8; data_size];
        let response = mk_response(name.clone(), value);
        responses.insert(name.to_string(), response.clone());
    }
    let mut current_tmp_dir = 0;
    for (name, packet) in &responses {
        let file = tmp_dirs[current_tmp_dir].join(name.clone());
        let mut f = fs::File::create(file).unwrap();
        let response_ser = bincode::serialize(&packet).unwrap();
        f.write_all(&response_ser).unwrap();
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
    let response = mk_response(name.clone().to_string(), vec![data; size]);
    let root_dir = generate_random_dir_name();
    let dir = root_dir.join(name);
    let mut f = fs::File::create(dir.clone()).unwrap();
    let response_ser = bincode::serialize(&response).unwrap();
    f.write_all(&response_ser).unwrap();
    f.sync_all().unwrap();
    root_dir.clone().to_string_lossy().to_string()
}

async fn single_fetch() {
    let size: usize = MB5;
    let network: Vec<Config> = vec![
        Config {
            listen_addr: "127.0.0.1:50100".parse().unwrap(),
            content_store_size: 50,
            peers: Some(vec!["127.0.0.1:50101".into()]),
            data_dir: populate_tmp_dir("hello0".to_string(), 0, 1024),
        },
        Config {
            listen_addr: "127.0.0.1:50101".parse().unwrap(),
            content_store_size: 50,
            peers: Some(vec!["127.0.0.1:50102".into()]),
            data_dir: populate_tmp_dir("hello1".to_string(), 1, 1024),
        },
        Config {
            listen_addr: "127.0.0.1:50102".parse().unwrap(),
            content_store_size: 50,
            peers: Some(vec!["127.0.0.1:50103".into()]),
            data_dir: populate_tmp_dir("hello2".to_string(), 2, 1024),
        },
        Config {
            listen_addr: "127.0.0.1:50103".parse().unwrap(),
            content_store_size: 50,
            peers: None,
            data_dir: populate_tmp_dir("hello3".to_string(), 3, size),
        }
    ];
    setup_network(network);
    std::thread::sleep(std::time::Duration::from_millis(1000));
    let mut cc = CopernicaRequestor::new("127.0.0.1:50099".into(), "127.0.0.1:50100".into());
    cc.start_polling();
    //let actual_hello0 = cc.request("hello2".to_string()).await;
    //let expected_hello0 = mk_response("hello2".to_string(), vec![2; 1024]);
    //assert_eq!(actual_hello0, Some(expected_hello0));

    if let Some(actual_hello0) = cc.request("hello0".to_string()).await {
        println!("MISSING {:?}", actual_hello0.missing());
    }
    //let expected_hello3 = mk_response("hello3".to_string(), vec![3; size]);
    //assert_eq!(actual_hello3, Some(expected_hello3));
}

async fn small_world_graph_lt_mtu() {
    // https://en.wikipedia.org/wiki/File:Small-world-network-example.png
    // node0 is 12 o'clock, node1 is 1 o'clock, etc.
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
                 data_dir: populate_tmp_dir("hello0".to_string(), 0, constants::FRAGMENT_SIZE ),
        },
        Config { listen_addr: "127.0.0.1:50001".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50000".into(),
                                  "127.0.0.1:50002".into()]),
                 data_dir: populate_tmp_dir("hello1".to_string(), 1, constants::FRAGMENT_SIZE ),
        },
        Config { listen_addr: "127.0.0.1:50002".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50000".into(),
                                  "127.0.0.1:50001".into(),
                                  "127.0.0.1:50003".into(),
                                  "127.0.0.1:50004".into()]),
                 data_dir: populate_tmp_dir("hello2".to_string(), 2, constants::FRAGMENT_SIZE ),
        },
        Config { listen_addr: "127.0.0.1:50003".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50000".into(),
                                  "127.0.0.1:50002".into(),
                                  "127.0.0.1:50004".into(),
                                  "127.0.0.1:50007".into()]),
                 data_dir: populate_tmp_dir("hello3".to_string(), 3, constants::FRAGMENT_SIZE ),
        },
        Config { listen_addr: "127.0.0.1:50004".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50002".into(),
                                  "127.0.0.1:50003".into(),
                                  "127.0.0.1:50005".into()]),
                 data_dir: populate_tmp_dir("hello4".to_string(), 4, constants::FRAGMENT_SIZE ),
        },
        Config { listen_addr: "127.0.0.1:50005".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50000".into(),
                                  "127.0.0.1:50004".into(),
                                  "127.0.0.1:50006".into()]),
                 data_dir: populate_tmp_dir("hello5".to_string(), 5, constants::FRAGMENT_SIZE ),
        },
        Config { listen_addr: "127.0.0.1:50006".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50005".into(),
                                  "127.0.0.1:50007".into(),
                                  "127.0.0.1:50008".into()]),
                 data_dir: populate_tmp_dir("hello6".to_string(), 6, constants::FRAGMENT_SIZE ),
        },
        Config { listen_addr: "127.0.0.1:50007".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50000".into(),
                                  "127.0.0.1:50003".into(),
                                  "127.0.0.1:50006".into(),
                                  "127.0.0.1:50008".into(),
                                  "127.0.0.1:50009".into(),
                                  "127.0.0.1:50010".into()]),
                 data_dir: populate_tmp_dir("hello7".to_string(), 7, constants::FRAGMENT_SIZE ),
        },
        Config { listen_addr: "127.0.0.1:50008".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50006".into(),
                                  "127.0.0.1:50007".into(),
                                  "127.0.0.1:50009".into()]),
                 data_dir: populate_tmp_dir("hello8".to_string(), 8, constants::FRAGMENT_SIZE ),
        },
        Config { listen_addr: "127.0.0.1:50009".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50007".into(),
                                  "127.0.0.1:50008".into(),
                                  "127.0.0.1:50010".into(),
                                  "127.0.0.1:50000".into()]),
                 data_dir: populate_tmp_dir("hello9".to_string(), 9, constants::FRAGMENT_SIZE ),
        },
        Config { listen_addr: "127.0.0.1:50010".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50007".into(),
                                  "127.0.0.1:50009".into(),
                                  "127.0.0.1:50011".into(),
                                  "127.0.0.1:50000".into()]),
                 data_dir: populate_tmp_dir("hello10".to_string(), 10, constants::FRAGMENT_SIZE ),
        },
        Config { listen_addr: "127.0.0.1:50011".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50010".into(),
                                  "127.0.0.1:50000".into()]),
                 data_dir: populate_tmp_dir("hello11".to_string(), 11, constants::FRAGMENT_SIZE ),
    }];
    setup_network(network);
    let mut cc = CopernicaRequestor::new("127.0.0.1:49999".into(), "127.0.0.1:50004".into());
    cc.start_polling();
    for n in 0..11 {
        let expected = mk_response(format!("hello{}", n), vec![n; constants::FRAGMENT_SIZE ]);
        let actual = cc.request(format!("hello{}", n)).await;
        assert_eq!(actual, Some(expected));
    }
}

async fn small_world_graph_gt_mtu() {
    // https://en.wikipedia.org/wiki/File:Small-world-network-example.png
    // node0 is 12 o'clock, node1 is 1 o'clock, etc.
    let size: usize = 1600;
    let tmp_dirs = populate_tmp_dir_dispersed_gt_mtu(12, size);
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
    std::thread::sleep(std::time::Duration::from_millis(1000));
    let mut cc = CopernicaRequestor::new("127.0.0.1:50019".into(), "127.0.0.1:50024".into());
    cc.start_polling();
    for n in 0..11 {
        let expected = mk_response(format!("hello{}", n), vec![n; size]);
        let actual = cc.request(format!("hello{}", n)).await;
        assert_eq!(actual, Some(expected));
    }

}
/*
async fn timeout() {
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
    cc.start_polling();
    let actual_hello0 = cc.request("hello0".to_string(), 50);
    let expected_hello1 = mk_response("hello1".to_string(), None);
    assert_eq!(actual, expected);
}
*/
async fn resolve_gt_mtu() {
    let size: usize = MB0_1;
    let network: Vec<Config> = vec![
        Config {
            listen_addr: "127.0.0.1:50106".parse().unwrap(),
            content_store_size: 50000,
            peers: None,
            data_dir: populate_tmp_dir("hello0".to_string(), 0, size),
        },
    ];
    setup_network(network);
    let mut cc = CopernicaRequestor::new("127.0.0.1:50105".into(), "127.0.0.1:50106".into());
    cc.start_polling();
    std::thread::sleep(std::time::Duration::from_millis(3000));
    let actual = cc.request("hello0".to_string()).await;
    std::thread::sleep(std::time::Duration::from_millis(1000)); // to print all faces outbound
    let expected: Response = mk_response("hello0".to_string(), vec![0; size]);
    assert_eq!(actual, Some(expected));
}

async fn resolve_lt_mtu() {
    let size: usize = 1;
    let network: Vec<Config> = vec![
        Config {
            listen_addr: "127.0.0.1:50107".parse().unwrap(),
            content_store_size: 50,
            peers: None,
            data_dir: populate_tmp_dir("hello".to_string(), 0, size),
        },
    ];
    setup_network(network);
    let mut cc = CopernicaRequestor::new("127.0.0.1:50098".into(), "127.0.0.1:50107".into());
    cc.start_polling();
    std::thread::sleep(std::time::Duration::from_millis(3));
    let actual = cc.request("hello".to_string()).await;
    std::thread::sleep(std::time::Duration::from_millis(3));
    let expected: Response = mk_response("hello".to_string(), vec![0; size]);
    assert_eq!(actual, Some(expected));
}

async fn resolve_gt_mtu_two_nodes() {
    //let size: usize = 1428;
    let size: usize = 1500;
    let network: Vec<Config> = vec![
        Config {
            listen_addr: "127.0.0.1:50109".parse().unwrap(),
            content_store_size: 1,
            peers: None,
            data_dir: populate_tmp_dir("ceo1q0te4aj3u2llwl4mxuxnjm9skj897hncanvgcnz0gf3x57ap6h7gk4dw8nv::hello0".to_string(), 0, size),
        },
        Config {
            listen_addr: "127.0.0.1:50108".parse().unwrap(),
            content_store_size: 1,
            peers: Some(vec!["127.0.0.1:50109".into()]),
            data_dir: generate_random_dir_name().into_os_string().into_string().unwrap(),
        },
    ];
    setup_network(network);
    let mut cc = CopernicaRequestor::new("127.0.0.1:50103".into(), "127.0.0.1:50108".into());
    cc.start_polling();
    std::thread::sleep(std::time::Duration::from_millis(20));
    let actual = cc.request("ceo1q0te4aj3u2llwl4mxuxnjm9skj897hncanvgcnz0gf3x57ap6h7gk4dw8nv::hello0".to_string()).await;
    std::thread::sleep(std::time::Duration::from_millis(3));
    let expected: Response = mk_response("ceo1q0te4aj3u2llwl4mxuxnjm9skj897hncanvgcnz0gf3x57ap6h7gk4dw8nv::hello0".to_string(), vec![0; size]);
    assert_eq!(actual, Some(expected));
}

async fn resolve_lt_mtu_two_nodes() {
    let size: usize = 1;
    let network: Vec<Config> = vec![
        Config {
            listen_addr: "127.0.0.1:50112".parse().unwrap(),
            content_store_size: 5000,
            peers: None,
            data_dir: populate_tmp_dir("hello0".to_string(), 0, size),
        },
        Config {
            listen_addr: "127.0.0.1:50111".parse().unwrap(),
            content_store_size: 5000,
            peers: Some(vec!["127.0.0.1:50112".into()]),
            data_dir: generate_random_dir_name().into_os_string().into_string().unwrap(),
        },
    ];
    setup_network(network);
    let mut cc = CopernicaRequestor::new("127.0.0.1:50110".into(), "127.0.0.1:50111".into());
    cc.start_polling();
    std::thread::sleep(std::time::Duration::from_millis(50));
    let actual = task::block_on(async { cc.request("hello0".to_string()).await });
    std::thread::sleep(std::time::Duration::from_millis(10));
    let expected: Response = mk_response("hello0".to_string(), vec![0; size]);
    assert_eq!(actual, Some(expected));
}

fn main() {
    logger::setup_logging(3, None).unwrap();
    task::block_on(async {
        //small_world_graph_lt_mtu().await;
        //resolve_gt_mtu_two_nodes().await;
        //small_world_graph_lt_mtu().await;
        //resolve_lt_mtu_two_nodes().await;
        //small_world_graph_gt_mtu().await;
        //resolve_lt_mtu().await;
        //resolve_gt_mtu().await;
        single_fetch().await;
    });
}

#[cfg(test)]
mod network_regressions {
    use super::*;
    use async_std;

    #[test]
    fn test_single_fetch() {
        task::block_on(async {
            single_fetch().await;
        })
    }
    #[test]
    fn test_small_world_graph_lt_mtu() {
        task::block_on(async {
            small_world_graph_lt_mtu().await;
        })
    }

    #[test]
    fn test_small_world_graph_gt_mtu() {
        task::block_on(async {
            small_world_graph_gt_mtu().await;
        })
    }

/*
    #[test]
    fn test_timeout() {
        timeout().await;
    }
*/
    #[test]
    fn test_resolve_gt_mtu() {
        task::block_on(async {
            resolve_gt_mtu().await;
        })
    }

    #[test]
    fn test_resolve_lt_mtu() {
        task::block_on(async {
            resolve_lt_mtu().await;
        })
    }

    #[test]
    fn test_resolve_gt_mtu_two_nodes() {
        task::block_on(async {
            resolve_gt_mtu_two_nodes().await;
        })
    }
}
