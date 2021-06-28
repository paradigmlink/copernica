#![allow(dead_code)]
use {

    anyhow::{Result},
    copernica_protocols::{Echo, Protocol},
    copernica_broker::{Broker},
    copernica_common::{LinkId, ReplyTo, PrivateIdentityInterface},
    copernica_links::{Link, MpscChannel, MpscCorruptor, UdpIp},
    log::{debug},
};
pub fn smoke_test() -> Result<()> {
    let mut broker0 = Broker::new();
    let mut broker1 = Broker::new();
    let echo_protocol_sid0 = PrivateIdentityInterface::new_key();
    let echo_protocol_sid1 = PrivateIdentityInterface::new_key();
    let mut echo_protocol0: Echo = Protocol::new(echo_protocol_sid0.clone(), "echo_protocol0");
    let mut echo_protocol1: Echo = Protocol::new(echo_protocol_sid1.clone(), "echo_protocol1");

    // echo_protocol0 to broker0
    let link_sid0 = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id0 = LinkId::link_with_type(link_sid0.clone(), None, ReplyTo::Mpsc);
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), None, ReplyTo::Mpsc);
    let mut link0: MpscChannel = Link::new(link_id0.clone(), "link0", broker0.peer_with_link(link_id0.clone())?)?;
    let mut link1: MpscChannel = Link::new(link_id1.clone(), "link1",echo_protocol0.peer_with_link(link_id0.clone())?)?;
    link0.female(link1.male());
    link1.female(link0.male());

    // broker0 to broker1
    let link_sid2 = PrivateIdentityInterface::new_key();
    let link_sid3 = PrivateIdentityInterface::new_key();
    let link_id2 = LinkId::link_with_type(link_sid2.clone(), Some(link_sid3.public_id()), ReplyTo::Mpsc);
    let link_id3 = LinkId::link_with_type(link_sid3.clone(), Some(link_sid2.public_id()), ReplyTo::Mpsc);
    let mut link2: MpscCorruptor = Link::new(link_id2.clone(), "link2", broker0.peer_with_link(link_id2.clone())?)?;
    let mut link3: MpscCorruptor = Link::new(link_id3.clone(), "link3", broker1.peer_with_link(link_id3.clone())?)?;
    link2.female(link3.male());
    link3.female(link2.male());

    // broker1 to echo_protocol1
    let link_sid4 = PrivateIdentityInterface::new_key();
    let link_sid5 = PrivateIdentityInterface::new_key();
    let address4 = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
    let address5 = ReplyTo::UdpIp("127.0.0.1:50003".parse()?);
    let link_id4 = LinkId::link_with_type(link_sid4.clone(), Some(link_sid5.public_id()), address4.clone());
    let link_id5 = LinkId::link_with_type(link_sid5.clone(), Some(link_sid4.public_id()), address5.clone());
    let link4: UdpIp = Link::new(link_id4.clone(), "link4", broker1.peer_with_link(link_id4.remote(address5)?)?)?;
    let link5: UdpIp = Link::new(link_id5.clone(), "link5", echo_protocol1.peer_with_link(link_id5.remote(address4)?)?)?;

    //let links: Vec<Box<dyn Link>> = vec![Box::new(link0), Box::new(link1), Box::new(link2), Box::new(link3), Box::new(link4), Box::new(link5)];
    //for link in links {
    //    link.run()?;
    //}
    echo_protocol0.run()?;    // echo0 service is connected to link0
    link0.run()?;    // link0 link is connected to link1
    link1.run()?;    // etc
    broker0.run()?;
    link2.run()?;
    link3.run()?;
    broker1.run()?;
    link4.run()?;
    link5.run()?;
    echo_protocol1.run()?;

    debug!("cleartext  : \"ping\"");
    let pong: String = echo_protocol1.unreliable_unordered_cleartext_ping(echo_protocol_sid0.public_id())?;
    debug!("cleartext  : {:?}", pong);

    debug!("cyphertext : \"ping\"");
    let pong: String = echo_protocol0.unreliable_unordered_cyphertext_ping(echo_protocol_sid1.public_id())?;
    debug!("cyphertext : {:?}", pong);
    Ok(())
}
#[cfg(test)]
mod copernicafs {
/*
    #[test]
    fn test_smoke_test() {
        task::block_on(async {
            let _r = smoke_test().await;
        })
    }
*/
}
