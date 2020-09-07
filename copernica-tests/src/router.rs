#![allow(dead_code)]
use {
/*
    copernica::{
        ReplyTo, HBFI,
    },
    //borsh::{BorshSerialize},
    async_std::{ task, },
    std::{
        fs,
        io::prelude::*,
        //path::PathBuf,
        //io::Write,
        thread::{spawn},
        //collections::HashMap,
    },
*/
    anyhow::{Result},
    crate::{
        common::{
            //generate_random_dir_name,
            populate_tmp_dir, TestData},
    },

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

/*
#[allow(dead_code)]
fn router(config: Config) -> Result<()> {
    let mut router = Router::new_with_config(config)?;
    spawn( move || {
        task::block_on(async {
            router.run().await?;
            Ok::<(), anyhow::Error>(())
        })?;
        Ok::<(), anyhow::Error>(())
    });
    Ok(())
}

#[allow(dead_code)]
async fn setup_network(network: Vec<Config>) -> Result<()> {
    for node in network {
        router(node)?;
    }
    Ok(())
}
*/
pub async fn fetch_from_self() -> Result<()> {
    let mut td = TestData::new();
    td.push(("0.txt".into(), 1, 1024));
    let (_expected_data_dir, _actual_data_dir) = populate_tmp_dir("namable".into(), "namable_id".into(), td).await?;
    Ok(())
}

/*
pub async fn single_fetch() -> Result<()> {
    let name0: String = "hello0".into();
    let name1: String = "hello1".into();
    let name2: String = "hello2".into();
    let name3: String = "hello3".into();
    let id0: String = "id0".into();
    let id1: String = "id1".into();
    let id2: String = "id2".into();
    let id3: String = "id3".into();
    let mut td0 = TestData::new();
    td0.push(("0.txt".into(), 1, 1024));
    let mut td1 = TestData::new();
    td1.push(("1.txt".into(), 2, 2048));
    let mut td2 = TestData::new();
    td2.push(("2.txt".into(), 3, 1025));
    let mut td3 = TestData::new();
    td3.push(("3.txt".into(), 4, 10));
    let (expected_data_dir0, actual_data_dir0) = populate_tmp_dir(name0.clone(), id0.clone(), td0).await?;
    let (_expected_data_dir1, actual_data_dir1) = populate_tmp_dir(name1.clone(), id1.clone(), td1).await?;
    let (_expected_data_dir2, actual_data_dir2) = populate_tmp_dir(name2.clone(), id2.clone(), td2).await?;
    let (_expected_data_dir3, actual_data_dir3) = populate_tmp_dir(name3.clone(), id3.clone(), td3).await?;

    let network: Vec<Config> = vec![
        Config {
            listen_addr: "127.0.0.1:50100".parse().unwrap(),
            content_store_size: 50,
            peers: Some(vec!["127.0.0.1:50101".into()]),
            data_dir: actual_data_dir0,
        },
        Config {
            listen_addr: "127.0.0.1:50101".parse().unwrap(),
            content_store_size: 50,
            peers: Some(vec!["127.0.0.1:50102".into()]),
            data_dir: actual_data_dir1,
        },
        Config {
            listen_addr: "127.0.0.1:50102".parse().unwrap(),
            content_store_size: 50,
            peers: Some(vec!["127.0.0.1:50103".into()]),
            data_dir: actual_data_dir2,
        },
        Config {
            listen_addr: "127.0.0.1:50103".parse().unwrap(),
            content_store_size: 50,
            peers: None,
            data_dir: actual_data_dir3,
        }
    ];
    setup_network(network).await?;
    let data_dir = generate_random_dir_name().await;
    let rs = sled::open(data_dir)?;
    let listen = ReplyTo::Udp("127.0.0.1:50099".parse()?);
    let remote = ReplyTo::Udp("127.0.0.1:50100".parse()?);
    let fs: FileSharer = Requestor::new(rs, listen, remote);
    //fs.start_polling();

    let hbfi0: HBFI = HBFI::new(&name0, &id0)?;
    let files = fs.file_names(hbfi0.clone())?;
    for file_name in files {
        let actual_file = fs.file(hbfi0.clone(), file_name.clone())?;
        let expected_file_path = expected_data_dir0.join(file_name);
        let mut expected_file = fs::File::open(&expected_file_path)?;
        let mut expected_buffer = Vec::new();
        expected_file.read_to_end(&mut expected_buffer)?;
        assert_eq!(actual_file, expected_buffer);
    }

    let actual_hello1 = fs.request("hello1".to_string())?;
    assert_eq!(actual_hello1, expected_hello1);

    let actual_hello2 = fs.request("hello2".to_string())?;
    assert_eq!(actual_hello2, expected_hello2);

    let actual_hello3 = fs.request("hello3".to_string())?;
    assert_eq!(actual_hello3, expected_hello3);

    Ok(())
}

pub async fn small_world_graph_lt_mtu() -> Result<()> {
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
                 data_dir: populate_tmp_dir("hello0".to_string(), 0, constants::FRAGMENT_SIZE as usize ).await?,
        },
        Config { listen_addr: "127.0.0.1:50001".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50000".into(),
                                  "127.0.0.1:50002".into()]),
                 data_dir: populate_tmp_dir("hello1".to_string(), 1, constants::FRAGMENT_SIZE as usize ).await?,
        },
        Config { listen_addr: "127.0.0.1:50002".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50000".into(),
                                  "127.0.0.1:50001".into(),
                                  "127.0.0.1:50003".into(),
                                  "127.0.0.1:50004".into()]),
                 data_dir: populate_tmp_dir("hello2".to_string(), 2, constants::FRAGMENT_SIZE as usize ).await?,
        },
        Config { listen_addr: "127.0.0.1:50003".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50000".into(),
                                  "127.0.0.1:50002".into(),
                                  "127.0.0.1:50004".into(),
                                  "127.0.0.1:50007".into()]),
                 data_dir: populate_tmp_dir("hello3".to_string(), 3, constants::FRAGMENT_SIZE as usize ).await?,
        },
        Config { listen_addr: "127.0.0.1:50004".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50002".into(),
                                  "127.0.0.1:50003".into(),
                                  "127.0.0.1:50005".into()]),
                 data_dir: populate_tmp_dir("hello4".to_string(), 4, constants::FRAGMENT_SIZE as usize ).await?,
        },
        Config { listen_addr: "127.0.0.1:50005".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50000".into(),
                                  "127.0.0.1:50004".into(),
                                  "127.0.0.1:50006".into()]),
                 data_dir: populate_tmp_dir("hello5".to_string(), 5, constants::FRAGMENT_SIZE as usize ).await?,
        },
        Config { listen_addr: "127.0.0.1:50006".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50005".into(),
                                  "127.0.0.1:50007".into(),
                                  "127.0.0.1:50008".into()]),
                 data_dir: populate_tmp_dir("hello6".to_string(), 6, constants::FRAGMENT_SIZE as usize ).await?,
        },
        Config { listen_addr: "127.0.0.1:50007".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50000".into(),
                                  "127.0.0.1:50003".into(),
                                  "127.0.0.1:50006".into(),
                                  "127.0.0.1:50008".into(),
                                  "127.0.0.1:50009".into(),
                                  "127.0.0.1:50010".into()]),
                 data_dir: populate_tmp_dir("hello7".to_string(), 7, constants::FRAGMENT_SIZE as usize ).await?,
        },
        Config { listen_addr: "127.0.0.1:50008".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50006".into(),
                                  "127.0.0.1:50007".into(),
                                  "127.0.0.1:50009".into()]),
                 data_dir: populate_tmp_dir("hello8".to_string(), 8, constants::FRAGMENT_SIZE as usize ).await?,
        },
        Config { listen_addr: "127.0.0.1:50009".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50007".into(),
                                  "127.0.0.1:50008".into(),
                                  "127.0.0.1:50010".into(),
                                  "127.0.0.1:50000".into()]),
                 data_dir: populate_tmp_dir("hello9".to_string(), 9, constants::FRAGMENT_SIZE as usize ).await?,
        },
        Config { listen_addr: "127.0.0.1:50010".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50007".into(),
                                  "127.0.0.1:50009".into(),
                                  "127.0.0.1:50011".into(),
                                  "127.0.0.1:50000".into()]),
                 data_dir: populate_tmp_dir("hello10".to_string(), 10, constants::FRAGMENT_SIZE as usize ).await?,
        },
        Config { listen_addr: "127.0.0.1:50011".parse().unwrap(), content_store_size: 50,
                 peers: Some(vec!["127.0.0.1:50010".into(),
                                  "127.0.0.1:50000".into()]),
                 data_dir: populate_tmp_dir("hello11".to_string(), 11, constants::FRAGMENT_SIZE as usize ).await?,
    }];
    setup_network(network).await?;
    let data_dir = generate_random_dir_name().await.into_os_string().into_string().unwrap();
    let mut cc = CopernicaRequestor::new("127.0.0.1:49999".into(), "127.0.0.1:50004".into(), &data_dir)?;
    let retries: u8 = 2;
    let timeout_per_retry: u64 = 1000;
    cc.start_polling();
    for n in 0..11 {
        let expected = mk_response(format!("hello{}", n), vec![n; constants::FRAGMENT_SIZE as usize ])?;
        let actual = cc.request(format!("hello{}", n))?;
        assert_eq!(actual, expected);
    }
    Ok(())
}

pub async fn small_world_graph_gt_mtu() -> Result<()> {
    // https://en.wikipedia.org/wiki/File:Small-world-network-example.png
    // node0 is 12 o'clock, node1 is 1 o'clock, etc.
    let size: usize = 2000;
    let cs_size: u64 = 350;
    let tmp_dirs = populate_tmp_dir_dispersed_gt_mtu(12, size).await?;
    let network: Vec<Config> = vec![
        Config { listen_addr: "127.0.0.1:50020".parse().unwrap(), content_store_size: cs_size,
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
        Config { listen_addr: "127.0.0.1:50021".parse().unwrap(), content_store_size: cs_size,
                 peers: Some(vec!["127.0.0.1:50020".into(),
                                  "127.0.0.1:50022".into()]),
                 data_dir: tmp_dirs[1].clone(),
        },
        Config { listen_addr: "127.0.0.1:50022".parse().unwrap(), content_store_size: cs_size,
                 peers: Some(vec!["127.0.0.1:50020".into(),
                                  "127.0.0.1:50021".into(),
                                  "127.0.0.1:50023".into(),
                                  "127.0.0.1:50024".into()]),
                 data_dir: tmp_dirs[2].clone(),
        },
        Config { listen_addr: "127.0.0.1:50023".parse().unwrap(), content_store_size: cs_size,
                 peers: Some(vec!["127.0.0.1:50020".into(),
                                  "127.0.0.1:50022".into(),
                                  "127.0.0.1:50024".into(),
                                  "127.0.0.1:50027".into()]),
                 data_dir: tmp_dirs[3].clone(),
        },
        Config { listen_addr: "127.0.0.1:50024".parse().unwrap(), content_store_size: cs_size,
                 peers: Some(vec!["127.0.0.1:50022".into(),
                                  "127.0.0.1:50023".into(),
                                  "127.0.0.1:50025".into()]),
                 data_dir: tmp_dirs[4].clone(),
        },
        Config { listen_addr: "127.0.0.1:50025".parse().unwrap(), content_store_size: cs_size,
                 peers: Some(vec!["127.0.0.1:50020".into(),
                                  "127.0.0.1:50024".into(),
                                  "127.0.0.1:50026".into()]),
                 data_dir: tmp_dirs[5].clone(),
        },
        Config { listen_addr: "127.0.0.1:50026".parse().unwrap(), content_store_size: cs_size,
                 peers: Some(vec!["127.0.0.1:50025".into(),
                                  "127.0.0.1:50027".into(),
                                  "127.0.0.1:50028".into()]),
                 data_dir: tmp_dirs[6].clone(),
        },
        Config { listen_addr: "127.0.0.1:50027".parse().unwrap(), content_store_size: cs_size,
                 peers: Some(vec!["127.0.0.1:50020".into(),
                                  "127.0.0.1:50023".into(),
                                  "127.0.0.1:50026".into(),
                                  "127.0.0.1:50028".into(),
                                  "127.0.0.1:50029".into(),
                                  "127.0.0.1:50030".into()]),
                 data_dir: tmp_dirs[7].clone(),
        },
        Config { listen_addr: "127.0.0.1:50028".parse().unwrap(), content_store_size: cs_size,
                 peers: Some(vec!["127.0.0.1:50026".into(),
                                  "127.0.0.1:50027".into(),
                                  "127.0.0.1:50029".into()]),
                 data_dir: tmp_dirs[8].clone(),
        },
        Config { listen_addr: "127.0.0.1:50029".parse().unwrap(), content_store_size: cs_size,
                 peers: Some(vec!["127.0.0.1:50027".into(),
                                  "127.0.0.1:50028".into(),
                                  "127.0.0.1:50030".into(),
                                  "127.0.0.1:50020".into()]),
                 data_dir: tmp_dirs[9].clone(),
        },
        Config { listen_addr: "127.0.0.1:50030".parse().unwrap(), content_store_size: cs_size,
                 peers: Some(vec!["127.0.0.1:50027".into(),
                                  "127.0.0.1:50029".into(),
                                  "127.0.0.1:50031".into(),
                                  "127.0.0.1:50020".into()]),
                 data_dir: tmp_dirs[10].clone(),
        },
        Config { listen_addr: "127.0.0.1:50031".parse().unwrap(), content_store_size: cs_size,
                 peers: Some(vec!["127.0.0.1:50030".into(),
                                  "127.0.0.1:50020".into()]),
                 data_dir: tmp_dirs[11].clone(),
    }];
    setup_network(network).await?;
    let data_dir = generate_random_dir_name().await.into_os_string().into_string().unwrap();
    let mut cc = CopernicaRequestor::new("127.0.0.1:50019".into(), "127.0.0.1:50024".into(), &data_dir)?;
    let retries: u8 = 2;
    let timeout_per_retry: u64 = 1000;
    cc.start_polling();
/*    let name: String = "hello10".into();
    let count: u8 = 10;
    let expected = mk_response(name.clone(), vec![count; size]);
    let actual = cc.request(name, retries, timeout_per_retry).await;
    assert_eq!(actual, expected);
*/
    for n in 0..11 {
        let name: String = format!("hello{}", n);
        let expected = mk_response(name.clone(), vec![n; size])?;
        let actual = cc.request(name.clone())?;
        assert_eq!(actual, expected.clone(), "\n=================\nTesting {}\n=================\n", name);
    }
    Ok(())

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
    let retries: u8 = 2;
    let timeout_per_retry: u64 = 1000;
    cc.start_polling();
    let actual_hello0 = cc.request("hello0".to_string(), 50);
    let expected_hello1 = mk_response("hello1".to_string(), None);
    assert_eq!(actual, expected);
}
*/
pub async fn resolve_gt_mtu() -> Result<()> {
    let size: usize = GT_MTU;
    let network: Vec<Config> = vec![
        Config {
            listen_addr: "127.0.0.1:50106".parse().unwrap(),
            content_store_size: 50000,
            peers: None,
            data_dir: populate_tmp_dir("hello0".to_string(), 0, size).await?,
        },
    ];
    setup_network(network).await?;
    let data_dir = generate_random_dir_name().await.into_os_string().into_string().unwrap();
    let mut cc = CopernicaRequestor::new("127.0.0.1:50105".into(), "127.0.0.1:50106".into(), &data_dir)?;
    let retries: u8 = 2;
    let timeout_per_retry: u64 = 1000;
    cc.start_polling();
    let expected: Response = mk_response("hello0".to_string(), vec![0; size])?;
    let actual = cc.request("hello0".to_string())?;
    assert_eq!(actual, expected);
    Ok(())
}

pub async fn resolve_lt_mtu() -> Result<()> {
    let size: usize = 1;
    let network: Vec<Config> = vec![
        Config {
            listen_addr: "127.0.0.1:50107".parse().unwrap(),
            content_store_size: 50,
            peers: None,
            data_dir: populate_tmp_dir("hello".to_string(), 0, size).await?,
        },
    ];
    setup_network(network).await?;
    let data_dir = generate_random_dir_name().await.into_os_string().into_string().unwrap();
    let mut cc = CopernicaRequestor::new("127.0.0.1:50098".into(), "127.0.0.1:50107".into(), &data_dir)?;
    let retries: u8 = 2;
    let timeout_per_retry: u64 = 1000;
    cc.start_polling();
    let expected: Response = mk_response("hello".to_string(), vec![0; size])?;
    let actual = cc.request("hello".to_string())?;
    assert_eq!(actual, expected);
    Ok(())
}

pub async fn resolve_gt_mtu_two_nodes() -> Result<()> {
    let size: usize = 1428;
    //let size: usize = 1500;
    let network: Vec<Config> = vec![
        Config {
            listen_addr: "127.0.0.1:50109".parse().unwrap(),
            content_store_size: 1,
            peers: None,
            data_dir: populate_tmp_dir("ceo1q0te4aj3u2llwl4mxuxnjm9skj897hncanvgcnz0gf3x57ap6h7gk4dw8nv::hello0".to_string(), 0, size).await?,
        },
        Config {
            listen_addr: "127.0.0.1:50108".parse().unwrap(),
            content_store_size: 1,
            peers: Some(vec!["127.0.0.1:50109".into()]),
            data_dir: generate_random_dir_name().await.into_os_string().into_string().unwrap(),
        },
    ];
    setup_network(network).await?;
    let data_dir = generate_random_dir_name().await.into_os_string().into_string().unwrap();
    let mut cc = CopernicaRequestor::new("127.0.0.1:50103".into(), "127.0.0.1:50108".into(), &data_dir)?;
    let retries: u8 = 2;
    let timeout_per_retry: u64 = 1000;
    cc.start_polling();
    let actual = cc.request("ceo1q0te4aj3u2llwl4mxuxnjm9skj897hncanvgcnz0gf3x57ap6h7gk4dw8nv::hello0".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(3));
    let expected: Response = mk_response("ceo1q0te4aj3u2llwl4mxuxnjm9skj897hncanvgcnz0gf3x57ap6h7gk4dw8nv::hello0".to_string(), vec![0; size])?;
    assert_eq!(actual, expected);
    Ok(())
}

pub async fn resolve_lt_mtu_two_nodes() -> Result<()> {
    let size: usize = 1;
    let network: Vec<Config> = vec![
        Config {
            listen_addr: "127.0.0.1:50112".parse().unwrap(),
            content_store_size: 5000,
            peers: None,
            data_dir: populate_tmp_dir("hello0".to_string(), 0, size).await?,
        },
        Config {
            listen_addr: "127.0.0.1:50111".parse().unwrap(),
            content_store_size: 5000,
            peers: Some(vec!["127.0.0.1:50112".into()]),
            data_dir: generate_random_dir_name().await.into_os_string().into_string().unwrap(),
        },
    ];
    setup_network(network).await?;
    let data_dir = generate_random_dir_name().await.into_os_string().into_string().unwrap();
    let mut cc = CopernicaRequestor::new("127.0.0.1:50110".into(), "127.0.0.1:50111".into(), &data_dir)?;
    let retries: u8 = 2;
    let timeout_per_retry: u64 = 1000;
    cc.start_polling();
    let actual = cc.request("hello0".to_string())?;
    let expected: Response = mk_response("hello0".to_string(), vec![0; size])?;
    assert_eq!(actual, expected);
    Ok(())
}

#[cfg(test)]
mod network_regressions {
    use super::*;

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
*/
