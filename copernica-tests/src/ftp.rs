#![allow(dead_code)]
use {
    anyhow::{Result},
    crate::common::{populate_tmp_dir, TestData, generate_random_dir_name },
    sled,
    std::{
        io::prelude::*,
        fs,
    },
    //copernica_protocols::{FTP, Manifest, FileManifest},
    copernica_services::{FTPService, FTPCommands},
    copernica_broker::{Broker},
    copernica_common::{HBFI, LinkId, ReplyTo},
    copernica_links::{Link, MpscChannel, //MpscCorruptor,
        UdpIp
    },
    log::{debug},
};

pub async fn smoke_test() -> Result<()> {
    let mut test_data0 = TestData::new();
    test_data0.push(("0.txt".into(), 0, 1024));
    let name0: String = "namable0".into();
    let id0: String = "namable_id0".into();
    let (raw_data_dir0, packaged_data_dir0) = populate_tmp_dir(name0.clone(), id0.clone(), test_data0).await?;

    let mut test_data1 = TestData::new();
    test_data1.push(("1.txt".into(), 1, 1024));
    let name1: String = "namable1".into();
    let id1: String = "namable_id1".into();
    let (raw_data_dir1, packaged_data_dir1) = populate_tmp_dir(name1.clone(), id1.clone(), test_data1).await?;

    let rs0 = sled::open(packaged_data_dir0)?;
    let rs1 = sled::open(packaged_data_dir1)?;
    let rs2 = sled::open(generate_random_dir_name().await)?;

    let mut b = Broker::new(rs2);
    let mut ftp0 = FTPService::new(rs0);
    let mut ftp1 = FTPService::new(rs1);

    let ftp0v_b_id = LinkId::listen("ftp0v_b".into(), ReplyTo::Mpsc);
    let ftp0_vb_id = LinkId::listen("ftp0_vb".into(), ReplyTo::Mpsc);
    let mut ftp0v_b_link: MpscChannel = Link::new(ftp0v_b_id.clone(), ftp0.peer_with_link(ftp0v_b_id)?)?;
    let mut ftp0_vb_link: MpscChannel = Link::new(ftp0_vb_id.clone(), b.peer(ftp0_vb_id)?)?;
    ftp0v_b_link.female(ftp0_vb_link.male());
    ftp0_vb_link.female(ftp0v_b_link.male());

    let ftp1_vb_address = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
    let ftp1v_b_address = ReplyTo::UdpIp("127.0.0.1:50003".parse()?);
    let ftp1_vb_id = LinkId::listen("ftp1_vb".into(), ftp1_vb_address.clone());
    let ftp1v_b_id = LinkId::listen("ftp1v_b".into(), ftp1v_b_address.clone());
    let ftp1_vb_link: UdpIp = Link::new(ftp1_vb_id.clone(), b.peer(ftp1_vb_id.remote(ftp1v_b_address))?)?;
    let ftp1v_b_link: UdpIp = Link::new(ftp1v_b_id.clone(), ftp1.peer_with_link(ftp1v_b_id.remote(ftp1_vb_address))?)?;

    let links: Vec<Box<dyn Link>> = vec![
        Box::new(ftp0_vb_link),
        Box::new(ftp0v_b_link),
        Box::new(ftp1_vb_link),
        Box::new(ftp1v_b_link)
    ];
    for link in links {
        link.run()?;
    }
    let (ftp0_c2p_tx, ftp0_p2c_rx) = ftp0.peer_with_client()?;
    let (ftp1_c2p_tx, ftp1_p2c_rx) = ftp1.peer_with_client()?;
    b.run()?;
    ftp0.run()?;
    ftp1.run()?;

    let hbfi0: HBFI = HBFI::new(&name0, &id0)?;
    let hbfi1: HBFI = HBFI::new(&name1, &id1)?;

    ftp1_c2p_tx.send(FTPCommands::RequestFileList(hbfi0.clone()))?;
    let files0 = ftp1_p2c_rx.recv();
    if let FTPCommands::ResponseFileList(Some(files)) = files0? {
        debug!("files 1: {:?}", files);
        for file_name in files {
            ftp1_c2p_tx.send(FTPCommands::RequestFile(hbfi0.clone(), file_name.clone()))?;
            if let FTPCommands::ResponseFile(Some(actual_file)) = ftp1_p2c_rx.recv()? {
                let expected_file_path = raw_data_dir0.join(file_name);
                let mut expected_file = fs::File::open(&expected_file_path)?;
                let mut expected_buffer = Vec::new();
                expected_file.read_to_end(&mut expected_buffer)?;
                assert_eq!(actual_file, expected_buffer);
            }
        }
    }

    ftp0_c2p_tx.send(FTPCommands::RequestFileList(hbfi1.clone()))?;
    let files1 = ftp0_p2c_rx.recv();
    if let FTPCommands::ResponseFileList(Some(files)) = files1? {
        debug!("files 0: {:?}", files);
        for file_name in files {
            ftp0_c2p_tx.send(FTPCommands::RequestFile(hbfi1.clone(), file_name.clone()))?;
            if let FTPCommands::ResponseFile(Some(actual_file)) = ftp0_p2c_rx.recv()? {
                let expected_file_path = raw_data_dir1.join(file_name);
                let mut expected_file = fs::File::open(&expected_file_path)?;
                let mut expected_buffer = Vec::new();
                expected_file.read_to_end(&mut expected_buffer)?;
                assert_eq!(actual_file, expected_buffer);
            }
        }
    }
    Ok(())
}
/*
pub async fn transports() -> Result<()> {
    let drop_hook = Box::new(move || {});

    let mut test_data0 = TestData::new();
    test_data0.push(("0.txt".into(), 2, 2024));
    let name0: String = "namable0".into();
    let id0: String = "namable_id0".into();
    let (raw_data_dir0, packaged_data_dir0) = populate_tmp_dir(name0.clone(), id0.clone(), test_data0).await?;

    let mut test_data1 = TestData::new();
    test_data1.push(("1.txt".into(), 1, 1024));
    let name1: String = "namable1".into();
    let id1: String = "namable_id1".into();
    let (raw_data_dir1, packaged_data_dir1) = populate_tmp_dir(name1.clone(), id1.clone(), test_data1).await?;

    let brs0 = generate_random_dir_name().await;
    let brs1 = generate_random_dir_name().await;

    let frs0 = sled::open(packaged_data_dir0)?;
    let brs0 = sled::open(brs0)?;
    let brs1 = sled::open(brs1)?;
    let frs1 = sled::open(packaged_data_dir1)?;

    let mut f0: FTP = Protocol::new(frs0, drop_hook.clone());
    let mut b0 = Broker::new(brs0);
    let mut b1 = Broker::new(brs1);
    let mut f1: FTP = Protocol::new(frs1, drop_hook);

    let lid0to1 = LinkId::listen(ReplyTo::Mpsc);
    let lid1to0 = LinkId::listen(ReplyTo::Mpsc);

    let lid1to2 = LinkId::listen(ReplyTo::Mpsc);
    let lid2to1 = LinkId::listen(ReplyTo::Mpsc);

    let lid2to3_address = ReplyTo::UdpIp("127.0.0.1:50000".parse()?);
    let lid3to2_address = ReplyTo::UdpIp("127.0.0.1:50001".parse()?);
    let lid2to3 = LinkId::listen(lid2to3_address.clone());
    let lid3to2 = LinkId::listen(lid3to2_address.clone());

    let mut mpscchannel0: MpscCorruptor = Link::new(lid0to1.clone(), f0.peer(lid0to1)?)?;
    let mut mpscchannel1: MpscCorruptor = Link::new(lid1to0.clone(), b0.peer(lid1to0)?)?;
    let mut mpscchannel2: MpscChannel   = Link::new(lid1to2.clone(), b0.peer(lid1to2)?)?;
    let mut mpscchannel3: MpscChannel   = Link::new(lid2to1.clone(), b1.peer(lid2to1)?)?;
    let udpip4:           UdpIp         = Link::new(lid2to3.clone(), b1.peer(lid2to3.remote(lid3to2_address))?)?;
    let udpip5:           UdpIp         = Link::new(lid3to2.clone(), f1.peer(lid3to2.remote(lid2to3_address))?)?;

    mpscchannel0.female(mpscchannel1.male());
    mpscchannel1.female(mpscchannel0.male());
    mpscchannel2.female(mpscchannel3.male());
    mpscchannel3.female(mpscchannel2.male());

    let links: Vec<Box<dyn Link>> = vec![
        Box::new(mpscchannel0),
        Box::new(mpscchannel1),
        Box::new(mpscchannel2),
        Box::new(mpscchannel3),
        Box::new(udpip4),
        Box::new(udpip5)
    ];
    for link in links {
        link.run()?;
    }

    f0.run()?;
    b0.run()?;
    b1.run()?;
    f1.run()?;

    let hbfi0: HBFI = HBFI::new(&name0, &id0)?;
    let hbfi1: HBFI = HBFI::new(&name1, &id1)?;

    let manifest1: Manifest = f0.manifest(hbfi1.clone())?;
    let manifest0: Manifest = f1.manifest(hbfi0.clone())?;
    debug!("manifest 0: {:?}", manifest0);
    debug!("manifest 1: {:?}", manifest1);

    let file_manifest0: FileManifest = f1.file_manifest(hbfi0.clone())?;
    let file_manifest1: FileManifest = f0.file_manifest(hbfi1.clone())?;
    debug!("file manifest 0: {:?}", file_manifest0);
    debug!("file manifest 1: {:?}", file_manifest1);

    let files0 = f1.file_names(hbfi0.clone())?;
    let files1 = f0.file_names(hbfi1.clone())?;
    debug!("files 0: {:?}", files0);
    debug!("files 1: {:?}", files1);

    for file_name in files0 {
        let actual_file = f1.file(hbfi0.clone(), file_name.clone())?;
        debug!("{:?}", actual_file);
        let expected_file_path = raw_data_dir0.join(file_name);
        let mut expected_file = fs::File::open(&expected_file_path)?;
        let mut expected_buffer = Vec::new();
        expected_file.read_to_end(&mut expected_buffer)?;
        assert_eq!(actual_file, expected_buffer);
    }
    for file_name in files1 {
        let actual_file = f0.file(hbfi1.clone(), file_name.clone())?;
        debug!("{:?}", actual_file);
        let expected_file_path = raw_data_dir1.join(file_name);
        let mut expected_file = fs::File::open(&expected_file_path)?;
        let mut expected_buffer = Vec::new();
        expected_file.read_to_end(&mut expected_buffer)?;
        assert_eq!(actual_file, expected_buffer);
    }
    Ok(())
}
*/
#[cfg(test)]
mod copernicafs {
    use super::*;
    use async_std::{ task, };

    #[test]
    fn test_smoke_test() {
        task::block_on(async {
            let _r = smoke_test().await;
        })
    }
}
