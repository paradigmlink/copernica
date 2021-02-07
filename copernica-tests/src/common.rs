#![allow(dead_code)]
use {
    std::{
        fs,
        env,
        path::PathBuf,
        io::Write,
    },
    copernica_common::{HBFI},
    anyhow::{Result},
    copernica_identity::{PrivateIdentity},
};

pub fn generate_random_dir_name() -> PathBuf {
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
