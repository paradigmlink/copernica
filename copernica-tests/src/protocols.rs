#![allow(dead_code)]
use {

    anyhow::{Result},
    crate::common::{generate_random_dir_name},
    sled,
    copernica_protocols::{Echo, Protocol},
    copernica_broker::{Broker},
    copernica_common::{LinkId, ReplyTo, PrivateIdentityInterface},
    copernica_links::{Link, MpscChannel, MpscCorruptor, UdpIp},
    log::{debug},
};
pub fn smoke_test() -> Result<()> {
    let rs0 = sled::open(generate_random_dir_name())?;
    let rs1 = sled::open(generate_random_dir_name())?;
    let cbrs0 = sled::open(generate_random_dir_name())?;
    let cbrs1 = sled::open(generate_random_dir_name())?;

    let mut cb0 = Broker::new(cbrs0);
    let mut cb1 = Broker::new(cbrs1);
    let mut fs0: Echo = Protocol::new();
    let mut fs1: Echo = Protocol::new();

    // service0 to broker0
    let lsid0 = PrivateIdentityInterface::new_key();
    let lsid1 = PrivateIdentityInterface::new_key();
    let lid0 = LinkId::listen(lsid0.clone(), Some(lsid1.public_id()), ReplyTo::Mpsc);
    let lid1 = LinkId::listen(lsid1.clone(), Some(lsid0.public_id()), ReplyTo::Mpsc);
    let mut link0: MpscChannel = Link::new(lid0.clone(), cb0.peer_with_link(lid0.clone())?)?;
    let mut link1: MpscChannel = Link::new(lid1.clone(), fs0.peer_with_link(rs0, lid0, lsid0.clone())?)?;
    link0.female(link1.male());
    link1.female(link0.male());

    // broker0 to broker1
    let lsid4 = PrivateIdentityInterface::new_key();
    let lsid5 = PrivateIdentityInterface::new_key();
    let lid4 = LinkId::listen(lsid4.clone(), Some(lsid5.public_id()), ReplyTo::Mpsc);
    let lid5 = LinkId::listen(lsid5.clone(), Some(lsid4.public_id()), ReplyTo::Mpsc);
    let mut link4: MpscCorruptor = Link::new(lid4.clone(), cb0.peer_with_link(lid4.clone())?)?;
    let mut link5: MpscCorruptor = Link::new(lid5.clone(), cb1.peer_with_link(lid5.clone())?)?;
    link4.female(link5.male());
    link5.female(link4.male());

    // broker1 to service1
    let lsid2 = PrivateIdentityInterface::new_key();
    let lsid3 = PrivateIdentityInterface::new_key();
    let lid2to3_address = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
    let lid3to2_address = ReplyTo::UdpIp("127.0.0.1:50003".parse()?);
    let lid2to3 = LinkId::listen(lsid2.clone(), Some(lsid3.public_id()), lid2to3_address.clone());
    let lid3to2 = LinkId::listen(lsid3.clone(), Some(lsid2.public_id()), lid3to2_address.clone());
    let link2: UdpIp = Link::new(lid2to3.clone(), cb1.peer_with_link(lid2to3.remote(lid3to2_address)?)?)?;
    let link3: UdpIp = Link::new(lid3to2.clone(), fs1.peer_with_link(rs1, lid3to2.remote(lid2to3_address)?, lsid1.clone())?)?;

    let links: Vec<Box<dyn Link>> = vec![Box::new(link0), Box::new(link1), Box::new(link2), Box::new(link3), Box::new(link4), Box::new(link5)];
    for link in links {
        link.run()?;
    }
    cb0.run()?;
    cb1.run()?;
    fs0.run()?;
    fs1.run()?;

    debug!("requesting manifest 0");
    let manifest0: String = fs1.echo(lsid0.public_id())?;
    debug!("manifest 0: {:?}", manifest0);

    debug!("requesting manifest 1");
    let manifest1: String = fs0.echo(lsid1.public_id())?;
    debug!("manifest 1: {:?}", manifest1);
    Ok(())
}
/*
pub async fn transports() -> Result<()> {
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

    let mut f0: FTP = Protocol::new(frs0);
    let mut b0 = Broker::new(brs0);
    let mut b1 = Broker::new(brs1);
    let mut f1: FTP = Protocol::new(frs1);

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
/*
    #[test]
    fn test_smoke_test() {
        task::block_on(async {
            let _r = smoke_test().await;
        })
    }
*/
}
