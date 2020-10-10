#![feature(total_cmp)]

extern crate lru;
extern crate rand_chacha;

#[macro_use]
extern crate serde_derive;
extern crate borsh;
extern crate sha3;

#[cfg(test)]
extern crate bitvec;

mod bloom_filter;
mod copernica;
pub mod copernica_constants;
pub mod bayes;
mod hbfi;
mod link;
mod packets;
mod router;
pub use crate::{
    copernica::Copernica,
    hbfi::{HBFI, BFI},
    link::{LinkId, Nonce, ReplyTo},
    packets::{Data, InterLinkPacket, NarrowWaist, WirePacket},
    router::Router,
    bayes::{Bayes, LinkWeight},
};
