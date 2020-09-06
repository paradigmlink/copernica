#![allow(dead_code)]
use {
    anyhow::{Result},
    crate::common::{populate_tmp_dir, TestData, generate_random_dir_name },
    sled,
    std::{
        io::prelude::*,
        fs,
    },
    client_libs::{
        Requestor, RelayNode, Manifest, FileManifest, FileSharer
    },
    copernica::{
        HBFI, Copernica, Link, ReplyTo
    },
    transport::{Transport, MpscChannel, MpscCorruptor },
    log::{debug},
};


pub async fn smoke_test() -> Result<()> {
    let mut test_data0 = TestData::new();
    test_data0.push(("1.txt".into(), 1, 1025));
    let name0: String = "namable0".into();
    let id0: String = "namable_id0".into();
    let (raw_data_dir0, packaged_data_dir0) = populate_tmp_dir(name0.clone(), id0.clone(), test_data0).await?;

    let mut test_data1 = TestData::new();
    test_data1.push(("5.txt".into(), 5, 1024));
    let name1: String = "namable1".into();
    let id1: String = "namable_id1".into();
    let (raw_data_dir1, packaged_data_dir1) = populate_tmp_dir(name1.clone(), id1.clone(), test_data1).await?;

    let rs0 = sled::open(packaged_data_dir0)?;
    let rs1 = sled::open(packaged_data_dir1)?;
    let mut c0 = Copernica::new();
    let mut c1 = Copernica::new();
    let lid0 = Link::new(ReplyTo::Mpsc);
    let lid1 = Link::new(ReplyTo::Mpsc);
    let mut mpscchannel0: MpscChannel = Transport::new(lid0.clone(), c0.peer(lid0)?)?;
    let mut mpscchannel1: MpscChannel = Transport::new(lid1.clone(), c1.peer(lid1)?)?;
    mpscchannel0.female(mpscchannel1.male());
    mpscchannel1.female(mpscchannel0.male());
    let ts0: Vec<Box<dyn Transport>> = vec![Box::new(mpscchannel0)];
    let ts1: Vec<Box<dyn Transport>> = vec![Box::new(mpscchannel1)];
    let mut fs0: FileSharer = Requestor::new(rs0);
    let mut fs1: FileSharer = Requestor::new(rs1);
    fs0.start(c0, ts0)?;
    fs1.start(c1, ts1)?;


    let hbfi0: HBFI = HBFI::new(&name0, &id0)?;
    let hbfi1: HBFI = HBFI::new(&name1, &id1)?;
    let manifest0: Manifest = fs1.manifest(hbfi0.clone())?;
    let manifest1: Manifest = fs0.manifest(hbfi1.clone())?;
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

pub async fn two_hops() -> Result<()> {
    let mut test_data0 = TestData::new();
    test_data0.push(("0.txt".into(), 0, 1024));
    let name0: String = "namable0".into();
    let id0: String = "namable_id0".into();
    let (raw_data_dir0, packaged_data_dir0) = populate_tmp_dir(name0.clone(), id0.clone(), test_data0).await?;

    let relay_data_dir = generate_random_dir_name().await;

    let mut test_data1 = TestData::new();
    test_data1.push(("1.txt".into(), 1, 1024));
    let name1: String = "namable1".into();
    let id1: String = "namable_id1".into();
    let (raw_data_dir1, packaged_data_dir1) = populate_tmp_dir(name1.clone(), id1.clone(), test_data1).await?;

    let rs0 = sled::open(packaged_data_dir0)?;
    let rsr = sled::open(relay_data_dir)?;
    let rs1 = sled::open(packaged_data_dir1)?;

    let mut c0 = Copernica::new();
    let mut cr = Copernica::new();
    let mut c1 = Copernica::new();

    let lid0to1 = Link::new(ReplyTo::Mpsc);
    let lid1to0 = Link::new(ReplyTo::Mpsc);
    let lid1to2 = Link::new(ReplyTo::Mpsc);
    let lid2to1 = Link::new(ReplyTo::Mpsc);

    let mut mpscchannel0: MpscCorruptor = Transport::new(lid0to1.clone(), c0.peer(lid0to1)?)?;
    let mut mpscchannel1: MpscCorruptor = Transport::new(lid1to0.clone(), cr.peer(lid1to0)?)?;
    let mut mpscchannel2: MpscChannel   = Transport::new(lid1to2.clone(), cr.peer(lid1to2)?)?;
    let mut mpscchannel3: MpscChannel   = Transport::new(lid2to1.clone(), c1.peer(lid2to1)?)?;

    mpscchannel0.female(mpscchannel1.male());
    mpscchannel1.female(mpscchannel0.male());
    mpscchannel2.female(mpscchannel3.male());
    mpscchannel3.female(mpscchannel2.male());

    let ts0: Vec<Box<dyn Transport>> = vec![Box::new(mpscchannel0)];
    let tsr: Vec<Box<dyn Transport>> = vec![Box::new(mpscchannel1), Box::new(mpscchannel2)];
    let ts1: Vec<Box<dyn Transport>> = vec![Box::new(mpscchannel3)];
    let mut fs0: FileSharer = Requestor::new(rs0);
    let mut rn : RelayNode  = Requestor::new(rsr);
    let mut fs1: FileSharer = Requestor::new(rs1);
    fs0.start(c0, ts0)?;
    rn.start(cr, tsr)?;
    fs1.start(c1, ts1)?;

    let hbfi0: HBFI = HBFI::new(&name0, &id0)?;
    let hbfi1: HBFI = HBFI::new(&name1, &id1)?;

    let manifest0: Manifest = fs1.manifest(hbfi0.clone())?;
    let manifest1: Manifest = fs0.manifest(hbfi1.clone())?;
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

#[cfg(test)]
mod copernicafs {
    use super::*;

    #[test]
    fn test_smoke_test() {
        task::block_on(async {
            smoke_test().await;
        })
    }
}
