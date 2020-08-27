#![allow(dead_code)]
mod router;
mod copernicafs;
mod common;
use {
    async_std::{ task, },
    anyhow::{Result},
};

fn main() -> Result<()> {
    logger::setup_logging(3, None).unwrap();
    task::block_on(async {
        let r =
        //router::resolve_gt_mtu_two_nodes().await;
        //router::small_world_graph_lt_mtu().await;
        //router::resolve_lt_mtu_two_nodes().await;
        //router::small_world_graph_gt_mtu().await;
        //router::resolve_lt_mtu().await;
        //router::resolve_gt_mtu().await;
        //router::fetch_from_self().await;
        //router::single_fetch().await;
        copernicafs::smoke_test().await;
        //copernicafs::single_file_less_than_fragment_size().await;
        if let Err(r) = r {
            println!("{}", r);
        }
    });
    Ok(())
}

