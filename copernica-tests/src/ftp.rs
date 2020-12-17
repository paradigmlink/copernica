#![allow(dead_code)]
use {
    anyhow::{Result},
    crate::common::{populate_tmp_dir, TestData, generate_random_dir_name },
    sled,
    std::{
        io::prelude::*,
        fs,
    },
    copernica_services::{FTPService, FTPCommands},
    copernica_broker::{Broker},
    copernica_common::{HBFI, LinkId, ReplyTo},
    copernica_links::{Link, MpscChannel,  UdpIp },
    copernica_identity::{PrivateIdentity, Seed},
    log::{debug},
};

pub async fn cleartext_response_cleartext_link() -> Result<()> {
    let mut rng = rand::thread_rng();

    let mut test_data0 = TestData::new();
    test_data0.push(("0.txt".into(), 0, 100));
    let name: String = "namable0".into();
    let request_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let (raw_data_dir, packaged_data_dir) = populate_tmp_dir(name.clone(), response_sid.clone(), test_data0).await?;

    let rs0 = sled::open(packaged_data_dir)?;
    let rs1 = sled::open(generate_random_dir_name().await)?;
    let brs = sled::open(generate_random_dir_name().await)?;

    let mut b = Broker::new(brs);
    let mut ftp0 = FTPService::new(rs0, response_sid.clone());
    let mut ftp1 = FTPService::new(rs1, request_sid.clone());

    let lnk_sid0 = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let lnk_sid1 = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let lnk_sid2 = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let lnk_sid3 = PrivateIdentity::from_seed(Seed::generate(&mut rng));

    //let mpscv_b_id = LinkId::listen(lnk_sid0.clone(), Some(lnk_sid1.public_id()), ReplyTo::Mpsc);
    //let mpsc_vb_id = LinkId::listen(lnk_sid1.clone(), Some(lnk_sid0.public_id()), ReplyTo::Mpsc);
    let mpscv_b_id = LinkId::listen(lnk_sid0.clone(), None, ReplyTo::Mpsc);
    let mpsc_vb_id = LinkId::listen(lnk_sid1.clone(), None, ReplyTo::Mpsc);
    let mut mpscv_b_link: MpscChannel = Link::new("lv_b".into(), mpscv_b_id.clone(), ftp0.peer_with_link(mpscv_b_id)?)?;
    let mut mpsc_vb_link: MpscChannel = Link::new("l_vb".into(), mpsc_vb_id.clone(), b.peer(mpsc_vb_id)?)?;
    mpscv_b_link.female(mpsc_vb_link.male());
    mpsc_vb_link.female(mpscv_b_link.male());

    let ftp_vb_address = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
    let ftpv_b_address = ReplyTo::UdpIp("127.0.0.1:50003".parse()?);
    //let ftp_vb_id = LinkId::listen(lnk_sid2.clone(), Some(lnk_sid3.public_id()), ftp_vb_address.clone());
    //let ftpv_b_id = LinkId::listen(lnk_sid3.clone(), Some(lnk_sid2.public_id()), ftpv_b_address.clone());
    let ftp_vb_id = LinkId::listen(lnk_sid2.clone(), None, ftp_vb_address.clone());
    let ftpv_b_id = LinkId::listen(lnk_sid3.clone(), None, ftpv_b_address.clone());
    let ftp_vb_link: UdpIp = Link::new("l_vb".into(), ftp_vb_id.clone(), b.peer(ftp_vb_id.remote(ftpv_b_address)?)?)?;
    let ftpv_b_link: UdpIp = Link::new("lv_b".into(), ftpv_b_id.clone(), ftp1.peer_with_link(ftpv_b_id.remote(ftp_vb_address)?)?)?;

    let links: Vec<Box<dyn Link>> = vec![
        Box::new(mpsc_vb_link),
        Box::new(mpscv_b_link),
        Box::new(ftp_vb_link),
        Box::new(ftpv_b_link)
    ];
    for link in links {
        link.run()?;
    }
    let (ftp1_c2p_tx, ftp1_p2c_rx) = ftp1.peer_with_client()?;
    b.run()?;
    ftp0.run()?;
    ftp1.run()?;


    //let hbfi0: HBFI = HBFI::new(Some(request_sid.public_id()), response_sid.public_id(), "app", "m0d", "fun", &name)?;
    let hbfi0: HBFI = HBFI::new(None, response_sid.public_id(), "app", "m0d", "fun", &name)?;

    debug!("\t\t\tclient-to-protocol");
    ftp1_c2p_tx.send(FTPCommands::RequestFileList(hbfi0.clone()))?;
    let files0 = ftp1_p2c_rx.recv();
    debug!("\t\t\tprotocol-to-client");
    if let FTPCommands::ResponseFileList(Some(files)) = files0? {
        debug!("\t\t\t\tfiles 0: {:?}", files);
        for file_name in files {
            debug!("\t\t\tclient-to-protocol");
            ftp1_c2p_tx.send(FTPCommands::RequestFile(hbfi0.clone(), file_name.clone()))?;
            if let FTPCommands::ResponseFile(Some(actual_file)) = ftp1_p2c_rx.recv()? {
                debug!("\t\t\tprotocol-to-client");
                let expected_file_path = raw_data_dir.join(file_name);
                let mut expected_file = fs::File::open(&expected_file_path)?;
                let mut expected_buffer = Vec::new();
                expected_file.read_to_end(&mut expected_buffer)?;
                debug!("\t\t\t\texpected_file {:?}", expected_file);
                assert_eq!(actual_file, expected_buffer);
            }
        }
    }
    Ok(())
}
pub async fn encrypted_response_encrypted_link() -> Result<()> {
    let mut rng = rand::thread_rng();

    let mut test_data0 = TestData::new();
    test_data0.push(("0.txt".into(), 0, 100));
    let name: String = "namable0".into();
    let request_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let (raw_data_dir, packaged_data_dir) = populate_tmp_dir(name.clone(), response_sid.clone(), test_data0).await?;

    let rs0 = sled::open(packaged_data_dir)?;
    let rs1 = sled::open(generate_random_dir_name().await)?;
    let brs = sled::open(generate_random_dir_name().await)?;

    let mut b = Broker::new(brs);
    let mut ftp0 = FTPService::new(rs0, response_sid.clone());
    let mut ftp1 = FTPService::new(rs1, request_sid.clone());

    let lnk_sid0 = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let lnk_sid1 = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let lnk_sid2 = PrivateIdentity::from_seed(Seed::generate(&mut rng));
    let lnk_sid3 = PrivateIdentity::from_seed(Seed::generate(&mut rng));

    let mpscv_b_id = LinkId::listen(lnk_sid0.clone(), Some(lnk_sid1.public_id()), ReplyTo::Mpsc);
    let mpsc_vb_id = LinkId::listen(lnk_sid1.clone(), Some(lnk_sid0.public_id()), ReplyTo::Mpsc);
    //let mpscv_b_id = LinkId::listen(lnk_sid0.clone(), None, ReplyTo::Mpsc);
    //let mpsc_vb_id = LinkId::listen(lnk_sid1.clone(), None, ReplyTo::Mpsc);
    let mut mpscv_b_link: MpscChannel = Link::new("lv_b".into(), mpscv_b_id.clone(), ftp0.peer_with_link(mpscv_b_id)?)?;
    let mut mpsc_vb_link: MpscChannel = Link::new("l_vb".into(), mpsc_vb_id.clone(), b.peer(mpsc_vb_id)?)?;
    mpscv_b_link.female(mpsc_vb_link.male());
    mpsc_vb_link.female(mpscv_b_link.male());

    let ftp_vb_address = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
    let ftpv_b_address = ReplyTo::UdpIp("127.0.0.1:50003".parse()?);
    let ftp_vb_id = LinkId::listen(lnk_sid2.clone(), Some(lnk_sid3.public_id()), ftp_vb_address.clone());
    let ftpv_b_id = LinkId::listen(lnk_sid3.clone(), Some(lnk_sid2.public_id()), ftpv_b_address.clone());
    //let ftp_vb_id = LinkId::listen(lnk_sid2.clone(), None, ftp_vb_address.clone());
    //let ftpv_b_id = LinkId::listen(lnk_sid3.clone(), None, ftpv_b_address.clone());
    let ftp_vb_link: UdpIp = Link::new("l_vb".into(), ftp_vb_id.clone(), b.peer(ftp_vb_id.remote(ftpv_b_address)?)?)?;
    let ftpv_b_link: UdpIp = Link::new("lv_b".into(), ftpv_b_id.clone(), ftp1.peer_with_link(ftpv_b_id.remote(ftp_vb_address)?)?)?;

    let links: Vec<Box<dyn Link>> = vec![
        Box::new(mpsc_vb_link),
        Box::new(mpscv_b_link),
        Box::new(ftp_vb_link),
        Box::new(ftpv_b_link)
    ];
    for link in links {
        link.run()?;
    }
    let (ftp1_c2p_tx, ftp1_p2c_rx) = ftp1.peer_with_client()?;
    b.run()?;
    ftp0.run()?;
    ftp1.run()?;


    let hbfi0: HBFI = HBFI::new(Some(request_sid.public_id()), response_sid.public_id(), "app", "m0d", "fun", &name)?;
    //let hbfi0: HBFI = HBFI::new(None, response_sid.public_id(), "app", "m0d", "fun", &name)?;

    debug!("\t\t\tclient-to-protocol");
    ftp1_c2p_tx.send(FTPCommands::RequestFileList(hbfi0.clone()))?;
    let files0 = ftp1_p2c_rx.recv();
    debug!("\t\t\tprotocol-to-client");
    if let FTPCommands::ResponseFileList(Some(files)) = files0? {
        debug!("\t\t\t\tfiles 0: {:?}", files);
        for file_name in files {
            debug!("\t\t\tclient-to-protocol");
            ftp1_c2p_tx.send(FTPCommands::RequestFile(hbfi0.clone(), file_name.clone()))?;
            if let FTPCommands::ResponseFile(Some(actual_file)) = ftp1_p2c_rx.recv()? {
                debug!("\t\t\tprotocol-to-client");
                let expected_file_path = raw_data_dir.join(file_name);
                let mut expected_file = fs::File::open(&expected_file_path)?;
                let mut expected_buffer = Vec::new();
                expected_file.read_to_end(&mut expected_buffer)?;
                debug!("\t\t\t\texpected_file {:?}", expected_file);
                assert_eq!(actual_file, expected_buffer);
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod copernicafs {
    use super::*;
    use async_std::{ task, };

    #[test]
    fn test_encrypted_response_encrypted_link() {
        task::block_on(async {
            let _r = encrypted_response_encrypted_link().await;
        })
    }
    #[test]
    fn test_cleartext_response_cleartext_link() {
        task::block_on(async {
            let _r = cleartext_response_cleartext_link().await;
        })
    }
}
