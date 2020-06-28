#![allow(dead_code)]
mod router;
mod copernicafs;
use {
    async_std::{ task, },
};

fn main() {
    logger::setup_logging(3, None).unwrap();
    task::block_on(async {
        //router::resolve_gt_mtu_two_nodes().await;
        //router::small_world_graph_lt_mtu().await;
        //router::resolve_lt_mtu_two_nodes().await;
        //router::small_world_graph_gt_mtu().await;
        //router::resolve_lt_mtu().await;
        //router::resolve_gt_mtu().await;
        router::single_fetch().await;
    });
}

