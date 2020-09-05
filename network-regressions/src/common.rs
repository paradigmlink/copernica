#![allow(dead_code)]
use {
    std::{
        fs,
        env,
        path::PathBuf,
        io::Write,
    },
    client_libs::{
        FilePacker,
    },
    anyhow::{Result},
};

pub async fn generate_random_dir_name() -> PathBuf {
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

pub type TestData = Vec<(PathBuf, u8, usize)>;

pub async fn populate_tmp_dir(name: String, id: String, test_data: TestData) -> Result<(PathBuf, PathBuf)> {
    let router_data_dir = generate_random_dir_name().await;
    let source_data_dir = generate_random_dir_name().await;
    for (path, data, size) in test_data {
        let dir = source_data_dir.join(path);
        let data = vec![data; size];
        let mut f = fs::File::create(dir.clone()).unwrap();
        f.write_all(&data).unwrap();
        f.sync_all().unwrap();
    }
    let packer: FilePacker = FilePacker::new(&source_data_dir, &router_data_dir, name, id)?;
    packer.publish()?;
    Ok((source_data_dir, router_data_dir))
}

async fn populate_tmp_dir_dispersed_gt_mtu(node_count: usize, data_size: u64) -> Result<Vec<(String, String)>> {
    let mut tmp_dirs: Vec<(PathBuf, PathBuf)> = Vec::with_capacity(node_count);
    for n in 0..node_count {
        let source_data_dir = generate_random_dir_name().await;
        let router_data_dir = generate_random_dir_name().await;
        tmp_dirs.push((source_data_dir.clone(), router_data_dir.clone()));
        let name = format!("hello{}", n.clone());
        let value = vec![n.clone() as u8; data_size as usize];

        let source_file_name = source_data_dir.join(name.clone());
        let mut source_file = fs::File::create(source_file_name).unwrap();
        source_file.write_all(&value).unwrap();
        source_file.sync_all().unwrap();
        let id = "namable".into();
        let _packer = FilePacker::new(&source_data_dir, &router_data_dir, name ,id)?;
        _packer.publish()?;
    }
    Ok(tmp_dirs.iter().map(|(s,r)| (s.to_string_lossy().to_string(), r.to_string_lossy().to_string())).collect::<Vec<(String, String)>>())
}
