use {
    scaffolding::{ group, single, Ordering, TestTree, setting, settings::Timeout},
    copernica_packets::{
        ReplyTo, LinkPacket,
        LinkId, NarrowWaistPacket, PublicIdentityInterface, PrivateIdentityInterface, HBFI,
    },
    std::{
        time::Duration,
    },
};
pub fn primitive_link_packet(ordering: Ordering) -> TestTree {
    group!(
        format!("Unit tests, ordering with {:?}", ordering),
        ordering,
        [
            setting!(Timeout(Duration::from_secs(5))),
            single!(|| { cleartext_link_packet_on_cleartext_request() }),
            single!(|| { cyphertext_link_packet_on_cleartext_request() }),
            single!(|| { cleartext_link_packet_on_cyphertext_request() }),
            single!(|| { cyphertext_link_packet_on_cyphertext_request() }),
            single!(|| { cleartext_link_packet_on_cleartext_response() }),
            single!(|| { cyphertext_link_packet_on_cleartext_response() }),
            single!(|| { cleartext_link_packet_on_cyphertext_response() }),
            single!(|| { cyphertext_link_packet_on_cyphertext_response() }),
        ]
    )
}
fn cleartext_link_packet_on_cleartext_request() {
    let response_sid = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let hbfi = HBFI::new(PublicIdentityInterface::Absent, response_sid.public_id(), "test", "test", "test", "test").unwrap();
    let nw = NarrowWaistPacket::request(hbfi).unwrap();
    let expected_lp = LinkPacket::new(ReplyTo::Mpsc, nw);
    let actual_lps = expected_lp.as_bytes(link_id1.clone()).unwrap();
    let (_pub_id, actual_lp) = LinkPacket::from_bytes(&actual_lps, link_id1).unwrap();
    assert_eq!(expected_lp, actual_lp);
}
fn cyphertext_link_packet_on_cleartext_request() {
    let response_sid = PrivateIdentityInterface::new_key();
    let link_sid0 = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id0 = LinkId::link_with_type(link_sid0.clone(), PublicIdentityInterface::new(link_sid1.public_id()), ReplyTo::Mpsc);
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), PublicIdentityInterface::new(link_sid0.public_id()), ReplyTo::Mpsc);
    let hbfi = HBFI::new(PublicIdentityInterface::Absent, response_sid.public_id(), "test", "test", "test", "test").unwrap();
    let nw = NarrowWaistPacket::request(hbfi).unwrap();
    let expected_lp = LinkPacket::new(ReplyTo::Mpsc, nw);
    let actual_lps = expected_lp.as_bytes(link_id1.clone()).unwrap();
    let (_pub_id, actual_lp) = LinkPacket::from_bytes(&actual_lps, link_id0).unwrap();
    assert_eq!(expected_lp, actual_lp);
}
fn cleartext_link_packet_on_cyphertext_request() {
    let request_sid = PrivateIdentityInterface::new_key();
    let response_sid = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let hbfi = HBFI::new(PublicIdentityInterface::new(request_sid.public_id()), response_sid.public_id(), "test", "test", "test", "test").unwrap();
    let nw = NarrowWaistPacket::request(hbfi).unwrap();
    let expected_lp = LinkPacket::new(ReplyTo::Mpsc, nw);
    let actual_lps = expected_lp.as_bytes(link_id1.clone()).unwrap();
    let (_pub_id, actual_lp) = LinkPacket::from_bytes(&actual_lps, link_id1).unwrap();
    assert_eq!(expected_lp, actual_lp);
}
fn cyphertext_link_packet_on_cyphertext_request() {
    let request_sid = PrivateIdentityInterface::new_key();
    let response_sid = PrivateIdentityInterface::new_key();
    let link_sid0 = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id0 = LinkId::link_with_type(link_sid0.clone(), PublicIdentityInterface::new(link_sid1.public_id()), ReplyTo::Mpsc);
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), PublicIdentityInterface::new(link_sid0.public_id()), ReplyTo::Mpsc);
    let hbfi = HBFI::new(PublicIdentityInterface::new(request_sid.public_id()), response_sid.public_id(), "test", "test", "test", "test").unwrap();
    let nw = NarrowWaistPacket::request(hbfi).unwrap();
    let expected_lp = LinkPacket::new(ReplyTo::Mpsc, nw);
    let actual_lps = expected_lp.as_bytes(link_id1.clone()).unwrap();
    let (_pub_id, actual_lp) = LinkPacket::from_bytes(&actual_lps, link_id0).unwrap();
    assert_eq!(expected_lp, actual_lp);
}
fn cleartext_link_packet_on_cleartext_response() {
    let response_sid = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let hbfi = HBFI::new(PublicIdentityInterface::Absent, response_sid.public_id(), "test", "test", "test", "test").unwrap();
    let nw = NarrowWaistPacket::response(response_sid, hbfi, "0123".as_bytes().to_vec()).unwrap();
    let expected_lp = LinkPacket::new(ReplyTo::Mpsc, nw);
    let actual_lps = expected_lp.as_bytes(link_id1.clone()).unwrap();
    let (_pub_id, actual_lp) = LinkPacket::from_bytes(&actual_lps, link_id1).unwrap();
    assert_eq!(expected_lp, actual_lp);
}
fn cyphertext_link_packet_on_cleartext_response() {
    let response_sid = PrivateIdentityInterface::new_key();
    let link_sid0 = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id0 = LinkId::link_with_type(link_sid0.clone(), PublicIdentityInterface::new(link_sid1.public_id()), ReplyTo::Mpsc);
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), PublicIdentityInterface::new(link_sid0.public_id()), ReplyTo::Mpsc);
    let hbfi = HBFI::new(PublicIdentityInterface::Absent, response_sid.public_id(), "test", "test", "test", "test").unwrap();
    let nw = NarrowWaistPacket::response(response_sid, hbfi, "0123".as_bytes().to_vec()).unwrap();
    let expected_lp = LinkPacket::new(ReplyTo::Mpsc, nw);
    let actual_lps = expected_lp.as_bytes(link_id1.clone()).unwrap();
    let (_pub_id, actual_lp) = LinkPacket::from_bytes(&actual_lps, link_id0).unwrap();
    assert_eq!(expected_lp, actual_lp);
}
fn cleartext_link_packet_on_cyphertext_response() {
    let request_sid = PrivateIdentityInterface::new_key();
    let response_sid = PrivateIdentityInterface::new_key();
    let link_sid0 = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id0 = LinkId::link_with_type(link_sid0.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let hbfi = HBFI::new(PublicIdentityInterface::new(request_sid.public_id()), response_sid.public_id(), "test", "test", "test", "test").unwrap();
    let nw = NarrowWaistPacket::response(response_sid, hbfi, "0123".as_bytes().to_vec()).unwrap();
    let expected_lp = LinkPacket::new(ReplyTo::Mpsc, nw);
    let actual_lps = expected_lp.as_bytes(link_id1.clone()).unwrap();
    let (_pub_id, actual_lp) = LinkPacket::from_bytes(&actual_lps, link_id0).unwrap();
    assert_eq!(expected_lp, actual_lp);
}
fn cyphertext_link_packet_on_cyphertext_response() {
    let request_sid = PrivateIdentityInterface::new_key();
    let response_sid = PrivateIdentityInterface::new_key();
    let link_sid0 = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id0 = LinkId::link_with_type(link_sid0.clone(), PublicIdentityInterface::new(link_sid1.public_id()), ReplyTo::Mpsc);
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), PublicIdentityInterface::new(link_sid0.public_id()), ReplyTo::Mpsc);
    let hbfi = HBFI::new(PublicIdentityInterface::new(request_sid.public_id()), response_sid.public_id(), "test", "test", "test", "test").unwrap();
    let nw = NarrowWaistPacket::response(response_sid, hbfi, "0123".as_bytes().to_vec()).unwrap();
    let expected_lp = LinkPacket::new(ReplyTo::Mpsc, nw);
    let actual_lps = expected_lp.as_bytes(link_id1.clone()).unwrap();
    let (_pub_id, actual_lp) = LinkPacket::from_bytes(&actual_lps, link_id0).unwrap();
    assert_eq!(expected_lp, actual_lp);
}
