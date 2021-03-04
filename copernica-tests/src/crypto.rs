#![allow(dead_code)]
use {
    anyhow::{Result},
    copernica_common::{HBFI, ReplyTo, NarrowWaistPacket, LinkPacket, LinkId, PrivateIdentityInterface},
    copernica_links::{encode, decode },
    std::net::{IpAddr, Ipv6Addr, SocketAddr},
};

pub fn encrypted_response_encrypted_link() -> Result<()> {
    let response_sid = PrivateIdentityInterface::new_key();
    let response_pid = response_sid.public_id();

    let request_sid = PrivateIdentityInterface::new_key();
    let request_pid = request_sid.public_id();
    let hbfi = HBFI::new(Some(request_pid), response_pid.clone(), "app", "m0d", "fun", "arg")?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;
    let expected_data = vec![0; 600];
    let offset = 100;
    let total = 100;
    let nw: NarrowWaistPacket = nw.transmute(response_sid.clone(), expected_data.clone(), offset, total)?;

    let lnk_tx_sid = PrivateIdentityInterface::new_key();
    let lnk_tx_pid = lnk_tx_sid.public_id();
    let lnk_rx_sid = PrivateIdentityInterface::new_key();
    let lnk_rx_pid = lnk_rx_sid.public_id();

    let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
    //let reply_to: ReplyTo = ReplyTo::Rf(32432);
    //let reply_to: ReplyTo = ReplyTo::Mpsc;
    let link_id_1 = LinkId::listen(lnk_tx_sid, Some(lnk_rx_pid), reply_to.clone());
    let link_id_2 = LinkId::listen(lnk_rx_sid, Some(lnk_tx_pid), reply_to.clone());
    let lp: LinkPacket = LinkPacket::new(reply_to, nw);
    let lps = encode(lp.clone(), link_id_1)?;
    let (_lnk_tx_pid, lpo) = decode(lps.clone(), link_id_2)?;


    let nw = lpo.narrow_waist();
    let actual_data = nw.data(Some(request_sid))?;

    assert_eq!(lpo, lp);
    assert_eq!(1461, lps.len());
    assert!(lps.len() <= 1472);
    assert_eq!(expected_data, actual_data);

    Ok(())
}

pub fn cleartext_response_encrypted_link() -> Result<()> {
    let response_sid = PrivateIdentityInterface::new_key();
    let response_pid = response_sid.public_id();

    let hbfi = HBFI::new(None, response_pid.clone(), "app", "m0d", "fun", "arg")?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;
    let expected_data = vec![0; 600];
    let offset = 100;
    let total = 100;
    let nw: NarrowWaistPacket = nw.transmute(response_sid.clone(), expected_data.clone(), offset, total)?;

    let lnk_tx_sid = PrivateIdentityInterface::new_key();
    let lnk_tx_pid = lnk_tx_sid.public_id();
    let lnk_rx_sid = PrivateIdentityInterface::new_key();
    let lnk_rx_pid = lnk_rx_sid.public_id();
    let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
    //let reply_to: ReplyTo = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
    //let reply_to: ReplyTo = ReplyTo::Rf(32432);
    //let reply_to: ReplyTo = ReplyTo::Mpsc;
    let link_id_1 = LinkId::listen(lnk_tx_sid, Some(lnk_rx_pid), reply_to.clone());
    let link_id_2 = LinkId::listen(lnk_rx_sid, Some(lnk_tx_pid), reply_to.clone());
    let lp: LinkPacket = LinkPacket::new(reply_to, nw);
    let lps = encode(lp.clone(), link_id_1)?;
    let (_lnk_tx_pid, lpo) = decode(lps.clone(), link_id_2)?;

    let nw = lpo.narrow_waist();
    let actual_data = nw.data(None)?;

    assert_eq!(lpo, lp);
    assert_eq!(1381, lps.len());
    assert!(lps.len() <= 1472);
    assert_eq!(expected_data, actual_data);

    Ok(())
}

pub fn encrypted_request_encrypted_link() -> Result<()> {
    let response_sid = PrivateIdentityInterface::new_key();
    let response_pid = response_sid.public_id();

    let request_sid = PrivateIdentityInterface::new_key();
    let request_pid = request_sid.public_id();

    let hbfi = HBFI::new(Some(request_pid), response_pid.clone(), "app", "m0d", "fun", "arg")?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;

    let lnk_tx_sid = PrivateIdentityInterface::new_key();
    let lnk_tx_pid = lnk_tx_sid.public_id();
    let lnk_rx_sid = PrivateIdentityInterface::new_key();
    let lnk_rx_pid = lnk_rx_sid.public_id();

    let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
    //let reply_to: ReplyTo = ReplyTo::Rf(32432);
    //let reply_to: ReplyTo = ReplyTo::Mpsc;
    let link_id_1 = LinkId::listen(lnk_tx_sid, Some(lnk_rx_pid), reply_to.clone());
    let link_id_2 = LinkId::listen(lnk_rx_sid, Some(lnk_tx_pid), reply_to.clone());
    let lp: LinkPacket = LinkPacket::new(reply_to, nw);
    let lps = encode(lp.clone(), link_id_1)?;
    let (_lnk_tx_pid, lpo) = decode(lps.clone(), link_id_2)?;


    assert_eq!(lpo, lp);
    assert_eq!(317, lps.len());
    assert!(lps.len() <= 1472);

    Ok(())
}

pub fn cleartext_request_encrypted_link() -> Result<()> {
    let response_sid = PrivateIdentityInterface::new_key();
    let response_pid = response_sid.public_id();


    let hbfi = HBFI::new(None, response_pid.clone(), "app", "m0d", "fun", "arg")?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;

    let lnk_tx_sid = PrivateIdentityInterface::new_key();
    let lnk_tx_pid = lnk_tx_sid.public_id();
    let lnk_rx_sid = PrivateIdentityInterface::new_key();
    let lnk_rx_pid = lnk_rx_sid.public_id();

    let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
    //let reply_to: ReplyTo = ReplyTo::Rf(32432);
    //let reply_to: ReplyTo = ReplyTo::Mpsc;
    let link_id_1 = LinkId::listen(lnk_tx_sid, Some(lnk_rx_pid), reply_to.clone());
    let link_id_2 = LinkId::listen(lnk_rx_sid, Some(lnk_tx_pid), reply_to.clone());
    let lp: LinkPacket = LinkPacket::new(reply_to, nw);
    let lps = encode(lp.clone(), link_id_1)?;
    let (_lnk_tx_pid, lpo) = decode(lps.clone(), link_id_2)?;
    assert_eq!(lpo, lp);
    assert_eq!(247, lps.len());
    assert!(lps.len() <= 1472);

    Ok(())
}

pub fn encrypted_response_cleartext_link() -> Result<()> {
    let response_sid = PrivateIdentityInterface::new_key();
    let response_pid = response_sid.public_id();

    let request_sid = PrivateIdentityInterface::new_key();
    let request_pid = request_sid.public_id();

    let hbfi = HBFI::new(Some(request_pid), response_pid.clone(), "app", "m0d", "fun", "arg")?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;
    let expected_data = vec![0; 600];
    let offset = 100;
    let total = 100;
    let nw: NarrowWaistPacket = nw.transmute(response_sid.clone(), expected_data.clone(), offset, total)?;

    let lnk_tx_sid = PrivateIdentityInterface::new_key();
    let lnk_rx_sid = PrivateIdentityInterface::new_key();

    let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
    //let reply_to: ReplyTo = ReplyTo::Rf(32432);
    //let reply_to: ReplyTo = ReplyTo::Mpsc;
    let link_id_1 = LinkId::listen(lnk_tx_sid, None, reply_to.clone());
    let link_id_2 = LinkId::listen(lnk_rx_sid, None, reply_to.clone());
    let lp: LinkPacket = LinkPacket::new(reply_to, nw);
    let lps = encode(lp.clone(), link_id_1)?;
    let (_lnk_tx_pid, lpo) = decode(lps.clone(), link_id_2)?;


    let nw = lpo.narrow_waist();
    let actual_data = nw.data(Some(request_sid))?;

    assert_eq!(lpo, lp);
    assert_eq!(1437, lps.len());
    assert!(lps.len() <= 1472);
    assert_eq!(expected_data, actual_data);

    Ok(())
}

pub fn cleartext_response_cleartext_link() -> Result<()> {
    let response_sid = PrivateIdentityInterface::new_key();
    let response_pid = response_sid.public_id();

    let hbfi = HBFI::new(None, response_pid.clone(), "app", "m0d", "fun", "arg")?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;
    let expected_data = vec![0; 600];
    let offset = 100;
    let total = 100;
    let nw: NarrowWaistPacket = nw.transmute(response_sid.clone(), expected_data.clone(), offset, total)?;

    let lnk_tx_sid = PrivateIdentityInterface::new_key();
    let lnk_rx_sid = PrivateIdentityInterface::new_key();
    //let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
    let reply_to: ReplyTo = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
    //let reply_to: ReplyTo = ReplyTo::Rf(32432);
    //let reply_to: ReplyTo = ReplyTo::Mpsc;
    let link_id_1 = LinkId::listen(lnk_tx_sid, None, reply_to.clone());
    let link_id_2 = LinkId::listen(lnk_rx_sid, None, reply_to.clone());
    let lp: LinkPacket = LinkPacket::new(reply_to, nw);
    let lps = encode(lp.clone(), link_id_1)?;
    let (_lnk_tx_pid, lpo) = decode(lps.clone(), link_id_2)?;

    let nw = lpo.narrow_waist();
    let actual_data = nw.data(None)?;

    assert_eq!(lpo, lp);
    assert_eq!(1345, lps.len());
    assert!(lps.len() <= 1472);
    assert_eq!(expected_data, actual_data);

    Ok(())
}

pub fn encrypted_request_cleartext_link() -> Result<()> {
    let response_sid = PrivateIdentityInterface::new_key();
    let response_pid = response_sid.public_id();

    let request_sid = PrivateIdentityInterface::new_key();
    let request_pid = request_sid.public_id();

    let hbfi = HBFI::new(Some(request_pid), response_pid.clone(), "app", "m0d", "fun", "arg")?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;

    let lnk_tx_sid = PrivateIdentityInterface::new_key();
    let lnk_rx_sid = PrivateIdentityInterface::new_key();

    let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
    //let reply_to: ReplyTo = ReplyTo::Rf(32432);
    //let reply_to: ReplyTo = ReplyTo::Mpsc;
    let link_id_1 = LinkId::listen(lnk_tx_sid, None, reply_to.clone());
    let link_id_2 = LinkId::listen(lnk_rx_sid, None, reply_to.clone());
    let lp: LinkPacket = LinkPacket::new(reply_to, nw);
    let lps = encode(lp.clone(), link_id_1)?;
    let (_lnk_tx_pid, lpo) = decode(lps.clone(), link_id_2)?;


    assert_eq!(lpo, lp);
    assert_eq!(293, lps.len());
    assert!(lps.len() <= 1472);

    Ok(())
}

pub fn cleartext_request_cleartext_link() -> Result<()> {
    let response_sid = PrivateIdentityInterface::new_key();
    let response_pid = response_sid.public_id();


    let hbfi = HBFI::new(None, response_pid.clone(), "app", "m0d", "fun", "arg")?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;

    let lnk_tx_sid = PrivateIdentityInterface::new_key();
    let lnk_rx_sid = PrivateIdentityInterface::new_key();

    let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
    //let reply_to: ReplyTo = ReplyTo::Rf(32432);
    //let reply_to: ReplyTo = ReplyTo::Mpsc;
    let link_id_1 = LinkId::listen(lnk_tx_sid, None, reply_to.clone());
    let link_id_2 = LinkId::listen(lnk_rx_sid, None, reply_to.clone());
    let lp: LinkPacket = LinkPacket::new(reply_to, nw);
    let lps = encode(lp.clone(), link_id_1)?;
    let (_lnk_tx_pid, lpo) = decode(lps.clone(), link_id_2)?;
    assert_eq!(lpo, lp);
    assert_eq!(223, lps.len());
    assert!(lps.len() <= 1472);

    Ok(())
}

pub fn request_transmute_and_decrypt() -> Result<()> {
    let response_sid = PrivateIdentityInterface::new_key();
    let response_pid = response_sid.public_id();
    let request_sid = PrivateIdentityInterface::new_key();
    let request_pid = request_sid.public_id();

    let hbfi = HBFI::new(Some(request_pid), response_pid.clone(), "app", "m0d", "fun", "arg")?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;
    let expected_data = vec![0; 600];
    let offset = 0;
    let total = 1;
    let nw: NarrowWaistPacket = nw.transmute(response_sid.clone(), expected_data.clone(), offset, total)?;
    let actual_data = nw.data(Some(request_sid))?;

    assert_eq!(actual_data, expected_data);
    Ok(())
}
/*
pub fn cleartext_response_encrypt_then_decrypt() -> Result<()> {
    let lnk_tx_sid = PrivateIdentityInterface::new_key();
    let response_sid = PrivateIdentityInterface::new_key();
    let response_pid = response_sid.public_id();
    let request_sid = PrivateIdentityInterface::new_key();
    let request_pid = request_sid.public_id();
    let false_sid = PrivateIdentityInterface::new_key();
    let false_pid = false_sid.public_id();

    let hbfi = HBFI::new(None, response_pid.clone(), "app", "m0d", "fun", "arg")?;
    let expected_data = vec![0; 10];
    let offset = 0;
    let total = 1;
    let nw: NarrowWaistPacket = NarrowWaistPacket::response(response_sid.clone(), hbfi.clone(), expected_data.clone(), offset, total)?;
    let hbfi = HBFI::new(Some(request_pid.clone()), response_pid.clone(), "app", "m0d", "fun", "arg")?;
    let hbfi = hbfi.cleartext_repr();
    let nw = nw.encrypt_for(response_sid, request_pid)?;

    let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
    let nw = lpo.narrow_waist();
    let actual_data = nw.data(Some(request_sid))?;
    println!("actual_data: {:?}", actual_data);


    assert_eq!(actual_data, expected_data);
    Ok(())
}
*/

#[cfg(test)]
mod crypto {
    use super::*;
    #[test]
    fn test_request_transmute_and_decrypt() {
        request_transmute_and_decrypt().unwrap();
    }
    #[test]
    fn test_encrypted_response_encrypted_link() {
        encrypted_response_encrypted_link().unwrap();
    }

    #[test]
    fn test_cleartext_response_encrypted_link() {
        cleartext_response_encrypted_link().unwrap();
    }
    #[test]
    fn test_encrypted_request_encrypted_link() {
        encrypted_request_encrypted_link().unwrap();
    }

    #[test]
    fn test_cleartext_request_encrypted_link() {
        cleartext_request_encrypted_link().unwrap();
    }

    #[test]
    fn test_encrypted_response_cleartext_link() {
        encrypted_response_cleartext_link().unwrap();
    }
    #[test]
    fn test_cleartext_response_cleartext_link() {
        cleartext_response_cleartext_link().unwrap();
    }

    #[test]
    fn test_encrypted_request_cleartext_link() {
        encrypted_request_cleartext_link().unwrap();
    }

    #[test]
    fn test_cleartext_request_cleartext_link() {
        cleartext_request_cleartext_link().unwrap();
    }
}
