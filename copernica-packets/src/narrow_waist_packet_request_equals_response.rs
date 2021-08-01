use {
    crate::{ NarrowWaistPacket },
    core::hash::{Hash, Hasher},
    std::{
        ops::{RangeBounds},
        cmp::Ordering,
        fmt,
    },
    //log::{error},
};
#[derive(Clone)]
pub struct NarrowWaistPacketReqEqRes(pub NarrowWaistPacket);
impl NarrowWaistPacketReqEqResBounds for NarrowWaistPacketReqEqRes {
    fn contains(&self, v: &NarrowWaistPacketReqEqRes) -> bool {
        let self_frm = match &self.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi.frm },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi.frm }
        };
        let other_frm = match &v.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi.frm },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi.frm }
        };
        self_frm == other_frm
    }
}
pub trait NarrowWaistPacketReqEqResBounds {
    fn contains(&self, v: &NarrowWaistPacketReqEqRes) -> bool;
}
impl<T> NarrowWaistPacketReqEqResBounds for T
where
    T: RangeBounds<NarrowWaistPacketReqEqRes>,
{
    fn contains(&self, v: &NarrowWaistPacketReqEqRes) -> bool {
        RangeBounds::contains(self, v)
    }
}
impl Hash for NarrowWaistPacketReqEqRes {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi.hash(state) },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi.hash(state) }
        }
    }
}
impl PartialOrd for NarrowWaistPacketReqEqRes {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_hbfi = match &self.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi }
        };
        let other_hbfi = match &other.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi }
        };
        Some(self_hbfi.cmp(other_hbfi))
    }
}

impl Ord for NarrowWaistPacketReqEqRes {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_hbfi = match &self.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi }
        };
        let other_hbfi = match &other.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi }
        };
        self_hbfi.frm.cmp(&other_hbfi.frm)
    }
}
impl PartialEq for NarrowWaistPacketReqEqRes {
    fn eq(&self, other: &Self) -> bool {
        let self_hbfi = match &self.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi }
        };
        let other_hbfi = match &other.0 {
            NarrowWaistPacket::Request { hbfi, .. } => { hbfi },
            NarrowWaistPacket::Response { hbfi, .. } => { hbfi }
        };
        self_hbfi == other_hbfi
    }
}
impl Eq for NarrowWaistPacketReqEqRes {}
impl fmt::Debug for NarrowWaistPacketReqEqRes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            NarrowWaistPacket::Request  { hbfi, .. } => write!(f, "NWEQ REQ {:?}", hbfi),
            NarrowWaistPacket::Response { hbfi, .. } => write!(f, "NWEQ RES {:?}", hbfi),
        }
    }
}
