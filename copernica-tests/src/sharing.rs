#![allow(dead_code)]
use {
    anyhow::{Result},
    crate::common::{populate_tmp_dir, TestData, generate_random_dir_name },
    sled,
    std::{
        io::prelude::*,
        fs,
    },
    copernica_services::{
        //RelayNode,
        Manifest, FileManifest, FileSharer
    },
    copernica_broker::{Broker},
    copernica_common::{
        HBFI, LinkId, ReplyTo
    },
    copernica_services::{Service},
    copernica_links::{Link, MpscChannel, //MpscCorruptor,
    UdpIp },
    log::{debug},
};

pub async fn smoke_test() -> Result<()> {
    let drop_hook = Box::new(move || {});

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

    let mut cb = Broker::new(rs2);
    let mut fs0: FileSharer = Service::new(rs0, drop_hook.clone());
    let mut fs1: FileSharer = Service::new(rs1, drop_hook);

    let lid0 = LinkId::listen(ReplyTo::Mpsc);
    let lid1 = LinkId::listen(ReplyTo::Mpsc);
    let mut mpscchannel0: MpscChannel = Link::new(lid0.clone(), cb.peer(lid0)?)?;
    let mut mpscchannel1: MpscChannel = Link::new(lid1.clone(), fs0.peer(lid1)?)?;
    mpscchannel0.female(mpscchannel1.male());
    mpscchannel1.female(mpscchannel0.male());

    let lid2to3_address = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
    let lid3to2_address = ReplyTo::UdpIp("127.0.0.1:50003".parse()?);
    let lid2to3 = LinkId::listen(lid2to3_address.clone());
    let lid3to2 = LinkId::listen(lid3to2_address.clone());
    let udpip2: UdpIp = Link::new(lid2to3.clone(), cb.peer(lid2to3.remote(lid3to2_address))?)?;
    let udpip3: UdpIp = Link::new(lid3to2.clone(), fs1.peer(lid3to2.remote(lid2to3_address))?)?;

    let links: Vec<Box<dyn Link>> = vec![Box::new(mpscchannel0), Box::new(mpscchannel1), Box::new(udpip2), Box::new(udpip3)];
    for link in links {
        link.run()?;
    }
    cb.run()?;
    fs0.run()?;
    fs1.run()?;

    let hbfi0: HBFI = HBFI::new(&name0, &id0)?;
    let hbfi1: HBFI = HBFI::new(&name1, &id1)?;
    debug!("requesting manifest 0");
    let manifest0: Manifest = fs1.manifest(hbfi0.clone())?;
    debug!("manifest 0: {:?}", manifest0);

    debug!("requesting manifest 1");
    let manifest1: Manifest = fs0.manifest(hbfi1.clone())?;
    debug!("manifest 1: {:?}", manifest1);

    debug!("requesting file manifest 0");
    let file_manifest0: FileManifest = fs1.file_manifest(hbfi0.clone())?;
    debug!("file manifest 0: {:?}", file_manifest0);

    debug!("requesting file manifest 1");
    let file_manifest1: FileManifest = fs0.file_manifest(hbfi1.clone())?;
    debug!("file manifest 1: {:?}", file_manifest1);
    debug!("requesting file names for 0");

    let files0 = fs1.file_names(hbfi0.clone())?;
    debug!("files 0: {:?}", files0);

    debug!("requesting file names for 0");
    let files1 = fs0.file_names(hbfi1.clone())?;
    debug!("files 1: {:?}", files1);

    for file_name in files0 {
        let actual_file = fs1.file(hbfi0.clone(), file_name.clone())?;
        debug!("{:?}", actual_file);
        let expected_file_path = raw_data_dir0.join(file_name);
        let mut expected_file = fs::File::open(&expected_file_path)?;
        let mut expected_buffer = Vec::new();
        expected_file.read_to_end(&mut expected_buffer)?;
        assert_eq!(actual_file, expected_buffer);
    }

    for file_name in files1 {
        let actual_file = fs0.file(hbfi1.clone(), file_name.clone())?;
        debug!("{:?}", actual_file);
        let expected_file_path = raw_data_dir1.join(file_name);
        let mut expected_file = fs::File::open(&expected_file_path)?;
        let mut expected_buffer = Vec::new();
        expected_file.read_to_end(&mut expected_buffer)?;
        assert_eq!(actual_file, expected_buffer);
    }
    Ok(())
}

/*
pub async fn smoke_test() -> Result<()> {
    let drop_hook = Box::new(move || {});

    let mut test_data0 = TestData::new();
    test_data0.push(("1.txt".into(), 1, 100025));
    let name0: String = "namable0".into();
    let id0: String = "namable_id0".into();
    let (raw_data_dir0, packaged_data_dir0) = populate_tmp_dir(name0.clone(), id0.clone(), test_data0).await?;


    let mut test_data1 = TestData::new();
    test_data1.push(("5.txt".into(), 5, 100024));
    let name1: String = "namable1".into();
    let id1: String = "namable_id1".into();
    let (raw_data_dir1, packaged_data_dir1) = populate_tmp_dir(name1.clone(), id1.clone(), test_data1).await?;

    let rs0 = sled::open(packaged_data_dir0)?;
    let rs1 = sled::open(packaged_data_dir1)?;
    let mut c0 = Broker::new();
    let mut c1 = Broker::new();
    let lid0 = LinkId::listen(ReplyTo::Mpsc);
    let lid1 = LinkId::listen(ReplyTo::Mpsc);
    let mut mpscchannel0: MpscChannel = Link::new(lid0.clone(), c0.peer(lid0)?)?;
    let mut mpscchannel1: MpscChannel = Link::new(lid1.clone(), c1.peer(lid1)?)?;
    mpscchannel0.female(mpscchannel1.male());
    mpscchannel1.female(mpscchannel0.male());
    let ts0: Vec<Box<dyn Link>> = vec![Box::new(mpscchannel0)];
    let ts1: Vec<Box<dyn Link>> = vec![Box::new(mpscchannel1)];
    let mut fs0: FileSharer = Service::new(rs0, drop_hook.clone());
    let mut fs1: FileSharer = Service::new(rs1, drop_hook);
    fs0.start(c0, ts0)?;
    fs1.start(c1, ts1)?;


    let hbfi0: HBFI = HBFI::new(&name0, &id0)?;
    let hbfi1: HBFI = HBFI::new(&name1, &id1)?;
    debug!("requesting manifest 0");
    let manifest0: Manifest = fs1.manifest(hbfi0.clone())?;
    debug!("manifest 0: {:?}", manifest0);

    debug!("requesting manifest 1");
    let manifest1: Manifest = fs0.manifest(hbfi1.clone())?;
    debug!("manifest 1: {:?}", manifest1);

    debug!("requesting file manifest 0");
    let file_manifest0: FileManifest = fs1.file_manifest(hbfi0.clone())?;
    debug!("file manifest 0: {:?}", file_manifest0);

    debug!("requesting file manifest 1");
    let file_manifest1: FileManifest = fs0.file_manifest(hbfi1.clone())?;
    debug!("file manifest 1: {:?}", file_manifest1);
    debug!("requesting file names for 0");

    let files0 = fs1.file_names(hbfi0.clone())?;
    debug!("files 0: {:?}", files0);

    debug!("requesting file names for 0");
    let files1 = fs0.file_names(hbfi1.clone())?;
    debug!("files 1: {:?}", files1);

    for file_name in files0 {
        let actual_file = fs1.file(hbfi0.clone(), file_name.clone())?;
        debug!("{:?}", actual_file);
        let expected_file_path = raw_data_dir0.join(file_name);
        let mut expected_file = fs::File::open(&expected_file_path)?;
        let mut expected_buffer = Vec::new();
        expected_file.read_to_end(&mut expected_buffer)?;
        assert_eq!(actual_file, expected_buffer);
    }

    for file_name in files1 {
        let actual_file = fs0.file(hbfi1.clone(), file_name.clone())?;
        debug!("{:?}", actual_file);
        let expected_file_path = raw_data_dir1.join(file_name);
        let mut expected_file = fs::File::open(&expected_file_path)?;
        let mut expected_buffer = Vec::new();
        expected_file.read_to_end(&mut expected_buffer)?;
        assert_eq!(actual_file, expected_buffer);
    }
    Ok(())
}

pub async fn transports() -> Result<()> {
    let drop_hook = Box::new(move || {});

    let mut test_data0 = TestData::new();
    test_data0.push(("0.txt".into(), 2, 2024));
    let name0: String = "namable0".into();
    let id0: String = "namable_id0".into();
    let (raw_data_dir0, packaged_data_dir0) = populate_tmp_dir(name0.clone(), id0.clone(), test_data0).await?;

    let relay_data_dir0 = generate_random_dir_name().await;
    let relay_data_dir1 = generate_random_dir_name().await;

    let mut test_data1 = TestData::new();
    test_data1.push(("1.txt".into(), 1, 1024));
    let name1: String = "namable1".into();
    let id1: String = "namable_id1".into();
    let (raw_data_dir1, packaged_data_dir1) = populate_tmp_dir(name1.clone(), id1.clone(), test_data1).await?;

    let rs0  = sled::open(packaged_data_dir0)?;
    let rsr0 = sled::open(relay_data_dir0)?;
    let rsr1 = sled::open(relay_data_dir1)?;
    let rs1  = sled::open(packaged_data_dir1)?;

    let mut c0 = Broker::new();
    let mut cr0 = Broker::new();
    let mut cr1 = Broker::new();
    let mut c1 = Broker::new();

    let lid0to1 = LinkId::listen(ReplyTo::Mpsc);
    let lid1to0 = LinkId::listen(ReplyTo::Mpsc);

    let lid1to2 = LinkId::listen(ReplyTo::Mpsc);
    let lid2to1 = LinkId::listen(ReplyTo::Mpsc);

    let lid2to3_address = ReplyTo::UdpIp("127.0.0.1:50000".parse()?);
    let lid3to2_address = ReplyTo::UdpIp("127.0.0.1:50001".parse()?);
    let lid2to3 = LinkId::listen(lid2to3_address.clone());
    let lid3to2 = LinkId::listen(lid3to2_address.clone());

    let mut mpscchannel0: MpscCorruptor = Link::new(lid0to1.clone(), c0.peer(lid0to1)?)?;
    let mut mpscchannel1: MpscCorruptor = Link::new(lid1to0.clone(), cr0.peer(lid1to0)?)?;
    let mut mpscchannel2: MpscChannel   = Link::new(lid1to2.clone(), cr0.peer(lid1to2)?)?;
    let mut mpscchannel3: MpscChannel   = Link::new(lid2to1.clone(), cr1.peer(lid2to1)?)?;
    let udpip4:           UdpIp         = Link::new(lid2to3.clone(), cr1.peer(lid2to3.remote(lid3to2_address))?)?;
    let udpip5:           UdpIp         = Link::new(lid3to2.clone(), c1.peer(lid3to2.remote(lid2to3_address))?)?;

    mpscchannel0.female(mpscchannel1.male());
    mpscchannel1.female(mpscchannel0.male());
    mpscchannel2.female(mpscchannel3.male());
    mpscchannel3.female(mpscchannel2.male());

    let ts0:  Vec<Box<dyn Link>> = vec![Box::new(mpscchannel0)];
    let tsr0: Vec<Box<dyn Link>> = vec![Box::new(mpscchannel1), Box::new(mpscchannel2)];
    let tsr1: Vec<Box<dyn Link>> = vec![Box::new(mpscchannel3), Box::new(udpip4)];
    let ts1:  Vec<Box<dyn Link>> = vec![Box::new(udpip5)];
    let mut fs0: FileSharer = Service::new(rs0, drop_hook.clone());
    let mut rn0: RelayNode  = Service::new(rsr0, drop_hook.clone());
    let mut rn1: RelayNode  = Service::new(rsr1, drop_hook.clone());
    let mut fs1: FileSharer = Service::new(rs1, drop_hook.clone());

    fs0.start(c0, ts0)?;
    rn0.start(cr0, tsr0)?;
    rn1.start(cr1, tsr1)?;
    fs1.start(c1, ts1)?;

    let hbfi0: HBFI = HBFI::new(&name0, &id0)?;
    let hbfi1: HBFI = HBFI::new(&name1, &id1)?;

    let manifest1: Manifest = fs0.manifest(hbfi1.clone())?;
    let manifest0: Manifest = fs1.manifest(hbfi0.clone())?;
    debug!("manifest 0: {:?}", manifest0);
    debug!("manifest 1: {:?}", manifest1);

    let file_manifest0: FileManifest = fs1.file_manifest(hbfi0.clone())?;
    let file_manifest1: FileManifest = fs0.file_manifest(hbfi1.clone())?;
    debug!("file manifest 0: {:?}", file_manifest0);
    debug!("file manifest 1: {:?}", file_manifest1);

    let files0 = fs1.file_names(hbfi0.clone())?;
    let files1 = fs0.file_names(hbfi1.clone())?;
    debug!("files 0: {:?}", files0);
    debug!("files 1: {:?}", files1);

    for file_name in files0 {
        let actual_file = fs1.file(hbfi0.clone(), file_name.clone())?;
        debug!("{:?}", actual_file);
        let expected_file_path = raw_data_dir0.join(file_name);
        let mut expected_file = fs::File::open(&expected_file_path)?;
        let mut expected_buffer = Vec::new();
        expected_file.read_to_end(&mut expected_buffer)?;
        assert_eq!(actual_file, expected_buffer);
    }
    for file_name in files1 {
        let actual_file = fs0.file(hbfi1.clone(), file_name.clone())?;
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
