#![allow(dead_code)]
use {
    anyhow::{Result},
    crate::common::{generate_random_dir_name, populate_tmp_dir, TestData },
    sled,
    std::{
        io::prelude::*,
        fs,
    },
    copernica::{
        client::{
            Requestor,
            file_sharing::{FileSharer, Manifest, FileManifest},
        },
        transport::{ReplyTo},
        hbfi::{HBFI},
    },
};

pub async fn packer_smoke_test() -> Result<()> {
    let mut test_data = TestData::new();
    test_data.push(("1.txt".into(), 1, 1024));
    test_data.push(("2.txt".into(), 2, 2048));
    test_data.push(("3.txt".into(), 3, 1025));
    test_data.push(("4.txt".into(), 4, 10));
    let name: String = "namable".into();
    let id: String = "namable_id".into();
    let (raw_data_dir, packaged_data_dir) = populate_tmp_dir(name.clone(), id.clone(), test_data).await?;
    let rs = sled::open(packaged_data_dir)?;
    let inbound  = ReplyTo::Udp("127.0.0.1:8089".parse()?);
    let outbound = ReplyTo::Udp("127.0.0.1:8090".parse()?);
    let mut fs: FileSharer = Requestor::new(rs, inbound, outbound);
    let hbfi: HBFI = HBFI::new(&name, &id)?;
    let manifest: Manifest = fs.manifest(hbfi.clone())?;
    let file_manifest: FileManifest = fs.file_manifest(hbfi.clone())?;
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
    fn test_packer_smoke_test() {
        task::block_on(async {
            packer_smoke_test().await;
        })
    }
}
