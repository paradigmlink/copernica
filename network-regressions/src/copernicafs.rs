#![allow(dead_code)]
use {
    anyhow::{Result},
    crate::common::{populate_tmp_dir, TestData },
    sled,
    std::{
        io::prelude::*,
        fs,
    },
    client_libs::{
        Requestor,
        file_sharing::{FileSharer, Manifest, FileManifest},
    },
    copernica::{
        ReplyTo, HBFI, Copernica, LinkId
    },
    transport::{Transport, MpscChannel},
};

pub async fn smoke_test() -> Result<()> {
    let mut test_data = TestData::new();
    test_data.push(("1.txt".into(), 1, 1024));
    test_data.push(("2.txt".into(), 2, 2048));
    test_data.push(("3.txt".into(), 3, 1025));
    test_data.push(("4.txt".into(), 4, 10));
    let name: String = "namable".into();
    let id: String = "namable_id".into();
    let (raw_data_dir, packaged_data_dir) = populate_tmp_dir(name.clone(), id.clone(), test_data).await?;

    let rs = sled::open(packaged_data_dir)?;
    let mut c = Copernica::new();
    let lid = LinkId::new(ReplyTo::Mpsc, 0);
    let udpip: MpscChannel = Transport::new(lid.clone(), c.create_link(lid)?)?;
    let ts: Vec<Box<dyn Transport>> = vec![Box::new(udpip)];
    let mut fs: FileSharer = Requestor::new(rs);
    fs.start(c, ts)?;

    let hbfi: HBFI = HBFI::new(&name, &id)?;
    let _manifest: Manifest = fs.manifest(hbfi.clone())?;
    let _file_manifest: FileManifest = fs.file_manifest(hbfi.clone())?;
    let files = fs.file_names(hbfi.clone())?;
    for file_name in files {
        let actual_file = fs.file(hbfi.clone(), file_name.clone())?;
        let expected_file_path = raw_data_dir.join(file_name);
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
