#![allow(dead_code)]
use {
    anyhow::{Result},
    copernica_common::{HBFI, ReplyTo, NarrowWaistPacket, LinkPacket},
    copernica_links::{encode, decode },
    copernica_identity::{PrivateIdentity, Seed},
    std::net::{IpAddr, Ipv6Addr, SocketAddr},
};

pub async fn encrypted_response_encrypted_link() -> Result<()> {
        let mut rng = rand::thread_rng();
        let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let response_pid = response_sid.public_id();

        let request_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let request_pid = request_sid.public_id();

        let hbfi = HBFI::new(response_pid.clone(), Some(request_pid), "app", "m0d", "fun", "arg")?;
        let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;
        let expected_data = vec![0; 600];
        let offset = 100;
        let total = 100;
        let nw: NarrowWaistPacket = nw.transmute(response_sid.clone(), expected_data.clone(), offset, total)?;

        let lnk_tx_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));

        let lnk_rx_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let lnk_rx_pid = lnk_rx_sid.public_id();

        let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
        //let reply_to: ReplyTo = ReplyTo::Rf(32432);
        //let reply_to: ReplyTo = ReplyTo::Mpsc;
        let lp: LinkPacket = LinkPacket::new(reply_to, nw);
        let lps = encode(lp.clone(), lnk_tx_sid, Some(lnk_rx_pid))?;
        let (_lnk_tx_pid, lpo) = decode(lps.clone(), Some(lnk_rx_sid))?;
        assert_eq!(lpo, lp);
        println!("handrolled length {} must be below 1472", lps.len());
        let lt1472 = if lps.len() <= 1472 { true } else { false };
        assert_eq!(true, lt1472);

        Ok(())
}

pub async fn cleartext_response_encrypted_link() -> Result<()> {
        let mut rng = rand::thread_rng();
        let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let response_pid = response_sid.public_id();

        let hbfi = HBFI::new(response_pid.clone(), None, "app", "m0d", "fun", "arg")?;
        let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;
        let expected_data = vec![0; 600];
        let offset = 100;
        let total = 100;
        let nw: NarrowWaistPacket = nw.transmute(response_sid.clone(), expected_data.clone(), offset, total)?;

        let lnk_tx_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));


        //let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
        let reply_to: ReplyTo = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
        //let reply_to: ReplyTo = ReplyTo::Rf(32432);
        //let reply_to: ReplyTo = ReplyTo::Mpsc;
        let lp: LinkPacket = LinkPacket::new(reply_to, nw);
        let lps = encode(lp.clone(), lnk_tx_sid, None)?;
        let (_lnk_tx_pid, lpo) = decode(lps.clone(), None)?;
        assert_eq!(lpo, lp);
        println!("handrolled length {} must be below 1472", lps.len());
        let lt1472 = if lps.len() <= 1472 { true } else { false };
        assert_eq!(true, lt1472);

        Ok(())
}

pub async fn encrypted_request_encrypted_link() -> Result<()> {
        let mut rng = rand::thread_rng();
        let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let response_pid = response_sid.public_id();

        let request_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let request_pid = request_sid.public_id();

        let hbfi = HBFI::new(response_pid.clone(), Some(request_pid), "app", "m0d", "fun", "arg")?;
        let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;
        let lnk_tx_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));

        let lnk_rx_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let lnk_rx_pid = lnk_rx_sid.public_id();

        let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
        //let reply_to: ReplyTo = ReplyTo::Rf(32432);
        //let reply_to: ReplyTo = ReplyTo::Mpsc;
        let lp: LinkPacket = LinkPacket::new(reply_to, nw);
        let lps = encode(lp.clone(), lnk_tx_sid, Some(lnk_rx_pid))?;
        let (_lnk_tx_pid, lpo) = decode(lps.clone(), Some(lnk_rx_sid))?;
        assert_eq!(lpo, lp);
        println!("handrolled length {} must be below 1472", lps.len());
        let lt1472 = if lps.len() <= 1472 { true } else { false };
        assert_eq!(true, lt1472);

        Ok(())
}

pub async fn cleartext_request_encrypted_link() -> Result<()> {
        let mut rng = rand::thread_rng();
        let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let response_pid = response_sid.public_id();


        let hbfi = HBFI::new(response_pid.clone(), None, "app", "m0d", "fun", "arg")?;
        let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone())?;

        let lnk_tx_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));


        let reply_to: ReplyTo = ReplyTo::UdpIp(SocketAddr::new(IpAddr::V6(Ipv6Addr::new(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535)), 65535));
        //let reply_to: ReplyTo = ReplyTo::Rf(32432);
        //let reply_to: ReplyTo = ReplyTo::Mpsc;
        let lp: LinkPacket = LinkPacket::new(reply_to, nw);
        let lps = encode(lp.clone(), lnk_tx_sid, None)?;
        let (_lnk_tx_pid, lpo) = decode(lps.clone(), None)?;
        assert_eq!(lpo, lp);
        println!("handrolled length {} must be below 1472", lps.len());
        let lt1472 = if lps.len() <= 1472 { true } else { false };
        assert_eq!(true, lt1472);

        Ok(())
}
