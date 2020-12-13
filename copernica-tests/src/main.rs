#![allow(dead_code)]
mod router;
mod common;
mod protocols;
mod ftp;
mod test_parse;
mod crypto;
use {
    async_std::{ task, },
    anyhow::{Result},
};

fn main() -> Result<()> {
    copernica_common::setup_logging(3, None).unwrap();
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
        //sharing::smoke_test().await;
        ftp::smoke_test().await;
        //crypto::encrypted_response_encrypted_link().await;
        //crypto::cleartext_response_encrypted_link().await;
        //crypto::encrypted_request_encrypted_link().await;
        //crypto::cleartext_request_encrypted_link().await;
        //crypto::request_transmute_and_decrypt().await;
        //crypto::cleartext_response_encrypt_then_decrypt().await;
        //protocols::transports().await;
        //copernicafs::single_file_less_than_fragment_size().await;
        if let Err(r) = r {
            println!("error {}", r);
        }
    });
    Ok(())
}

