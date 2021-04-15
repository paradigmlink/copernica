#![allow(dead_code)]
mod router;
mod common;
mod protocols;
mod crypto;
use {
    anyhow::{Result},
    log::{error},
};

fn main() -> Result<()> {
    copernica_common::setup_logging(3, None).unwrap();
    let r =
    //router::resolve_gt_mtu_two_nodes();
    //router::small_world_graph_lt_mtu();
    //router::resolve_lt_mtu_two_nodes();
    //router::small_world_graph_gt_mtu();
    //router::resolve_lt_mtu();
    //router::resolve_gt_mtu();
    //router::fetch_from_self();
    //router::single_fetch();
    //sharing::smoke_test();
    //ftp::encrypted_response_encrypted_link();
    //ftp::cleartext_response_cleartext_link();
    //crypto::encrypted_response_encrypted_link();
    //crypto::cleartext_response_encrypted_link();
    //crypto::encrypted_request_encrypted_link();
    //crypto::cleartext_request_encrypted_link();
    //crypto::encrypted_response_cleartext_link();
    //crypto::cleartext_response_cleartext_link();
    //crypto::encrypted_request_cleartext_link();
    //crypto::cleartext_request_cleartext_link();
    //crypto::request_transmute_and_decrypt();
    //crypto::cleartext_response_encrypt_then_decrypt();
    protocols::smoke_test();
    //copernicafs::single_file_less_than_fragment_size();
    match r {
        Ok(_) => println!("successful"),
        Err(r) =>  error!("{}", r)
    }
    Ok(())
}

