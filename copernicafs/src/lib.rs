use {
    anyhow::{ Result },
    log::{trace},
    copernica::{
        CopernicaRequestor,
        narrow_waist::{Bytes},
        response_store::{Response, mk_response},
    },
    logger::setup_logging,
    nix::{
        unistd,
        sys::stat,
    },
    async_std::{ task, },
    dirs,
    tempdir::TempDir,
    std::{
        fs::File,
        io::{BufRead, BufReader},
    },
    crossbeam_channel::{
        unbounded,
        Sender,
        Receiver,
        select,
        after,
        never
    },
};

#[derive(Clone)]
pub struct CopernicaFs {}

impl CopernicaFs {
    pub fn new() -> Self {
        Self{}
    }
}

