#![allow(dead_code)]
use {
    anyhow::{Result},
    crate::common::{generate_random_dir_name},
    copernica::{
        packer::{Packer},
    },
};

pub async fn single_file_less_than_fragment_size() -> Result<()> {
    let src_dir = generate_random_dir_name().await;
    let dest_dir = generate_random_dir_name().await;
    let p: Packer = Packer::new(&src_dir, &dest_dir)?;
    p.publish()?;

    println!("source: {:?}, dest{:?}", src_dir, dest_dir);
    Ok(())
}

#[cfg(test)]
mod copernicafs {
    use super::*;

    #[test]
    fn test_single_file_less_than_fragment_size() {
        task::block_on(async {
            single_file_less_than_fragment_size().await;
        })
    }
}
