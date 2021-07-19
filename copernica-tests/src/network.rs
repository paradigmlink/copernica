use {
    anyhow::{Result, anyhow},
    copernica_protocols::{Echo, Protocol},
    copernica_common::{LinkId, ReplyTo, PrivateIdentityInterface, PublicIdentityInterface, Operations, LogEntry},
    copernica_broker::{Broker},
    copernica_links::{Link, MpscChannel, MpscCorruptor, UdpIp, Corruption},
    crate::process_network,
    scaffolding::{ group, single, Ordering, TestTree, setting, settings::Timeout},
    std::{
        time::Duration,
        sync::mpsc::{channel},
        collections::HashMap,
    },
};
pub fn network_echo(ordering: Ordering) -> TestTree {
    group!(
        format!("Unit tests, ordering with {:?}", ordering),
        ordering,
        [
            setting!(Timeout(Duration::from_secs(2))),
            single!(|| { ping_pong() }),
        ]
    )
}
pub fn ping_pong() -> Result<()> {
    let router_0 = "router0";
    let router_1 = "router1";
    let echo_protocol_0 = "echo_protocol0";
    let echo_protocol_1 = "echo_protocol1";
    let link_0 = "link0";
    let link_1 = "link1";
    let link_2 = "link2";
    let link_3 = "link3";
    let link_4 = "link4";
    let link_5 = "link5";
    let (sender, receiver) = channel::<LogEntry>();
    let actual_behaviour = Operations::turned_on(sender);
    let mut broker0 = Broker::new(actual_behaviour.label(router_0.clone()));
    let mut broker1 = Broker::new(actual_behaviour.label(router_1.clone()));
    let echo_protocol_sid0 = PrivateIdentityInterface::new_key();
    let echo_protocol_sid1 = PrivateIdentityInterface::new_key();
    let mut echo_protocol0: Echo = Protocol::new(echo_protocol_sid0.clone(), actual_behaviour.label(echo_protocol_0.clone()));
    let mut echo_protocol1: Echo = Protocol::new(echo_protocol_sid1.clone(), actual_behaviour.label(echo_protocol_1.clone()));
    // echo_protocol0 to broker0
    let link_sid0 = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id0 = LinkId::link_with_type(link_sid0.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let mut link0: MpscChannel = Link::new(link_id0.clone(), actual_behaviour.label(link_0.clone()), broker0.peer_with_link(link_id0.clone())?)?;
    let mut link1: MpscChannel = Link::new(link_id1.clone(), actual_behaviour.label(link_1.clone()), echo_protocol0.peer_with_link(link_id0.clone())?)?;
    link0.female(link1.male());
    link1.female(link0.male());
    // broker0 to broker1
    let link_sid2 = PrivateIdentityInterface::new_key();
    let link_sid3 = PrivateIdentityInterface::new_key();
    let link_id2 = LinkId::link_with_type(link_sid2.clone(), PublicIdentityInterface::new(link_sid3.public_id()), ReplyTo::Mpsc);
    let link_id3 = LinkId::link_with_type(link_sid3.clone(), PublicIdentityInterface::new(link_sid2.public_id()), ReplyTo::Mpsc);
    let mut link2: MpscCorruptor = Link::new(link_id2.clone(), actual_behaviour.label(link_2.clone()), broker0.peer_with_link(link_id2.clone())?)?;
    let mut link3: MpscCorruptor = Link::new(link_id3.clone(), actual_behaviour.label(link_3.clone()), broker1.peer_with_link(link_id3.clone())?)?;
    link2.female(link3.male());
    link3.female(link2.male());
    // to echo_protocol1
    let link_sid4 = PrivateIdentityInterface::new_key();
    let link_sid5 = PrivateIdentityInterface::new_key();
    let address4 = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
    let address5 = ReplyTo::UdpIp("127.0.0.1:50003".parse()?);
    let link_id4 = LinkId::link_with_type(link_sid4.clone(), PublicIdentityInterface::new(link_sid5.public_id()), address4.clone());
    let link_id5 = LinkId::link_with_type(link_sid5.clone(), PublicIdentityInterface::new(link_sid4.public_id()), address5.clone());
    let mut link4: UdpIp = Link::new(link_id4.clone(), actual_behaviour.label(link_4.clone()), broker1.peer_with_link(link_id4.remote(address5)?)?)?;
    let mut link5: UdpIp = Link::new(link_id5.clone(), actual_behaviour.label(link_5.clone()), echo_protocol1.peer_with_link(link_id5.remote(address4)?)?)?;
    let mut expected_behaviour: HashMap<LogEntry, i32> = HashMap::new();
    expected_behaviour.insert(LogEntry::register(router_0.clone()), 1);
    expected_behaviour.insert(LogEntry::register(router_1.clone()), 1);
    expected_behaviour.insert(LogEntry::register(echo_protocol_0.clone()), 1);
    expected_behaviour.insert(LogEntry::register(echo_protocol_1.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_0.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_1.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_2.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_3.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_4.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_5.clone()), 1);
/*
    link3.corrupt(Corruption::Immune);
    expected_behaviour.insert(LogEntry::message(echo_protocol_0.clone()), 8);
    expected_behaviour.insert(LogEntry::message(echo_protocol_1.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_0.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_1.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_2.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_3.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_4.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_5.clone()), 8);
    expected_behaviour.insert(LogEntry::message(router_0.clone()), 16);
    expected_behaviour.insert(LogEntry::message(router_1.clone()), 16);
    expected_behaviour.insert(LogEntry::found_response_upstream(echo_protocol_0.clone()), 4);
    expected_behaviour.insert(LogEntry::found_response_upstream(echo_protocol_1.clone()), 0);
    expected_behaviour.insert(LogEntry::response_arrived_downstream(echo_protocol_0.clone()), 0);
    expected_behaviour.insert(LogEntry::response_arrived_downstream(echo_protocol_1.clone()), 4);
    expected_behaviour.insert(LogEntry::forward_response_downstream(router_0.clone()), 4);
    expected_behaviour.insert(LogEntry::forward_response_downstream(router_1.clone()), 4);
    expected_behaviour.insert(LogEntry::forward_request_upstream(router_0.clone()), 4);
    expected_behaviour.insert(LogEntry::forward_request_upstream(router_1.clone()), 4);

    link3.corrupt(Corruption::Integrity);
    expected_behaviour.insert(LogEntry::message(echo_protocol_0.clone()), 0);
    expected_behaviour.insert(LogEntry::message(echo_protocol_1.clone()), 4);
    expected_behaviour.insert(LogEntry::message(link_0.clone()), 0);
    expected_behaviour.insert(LogEntry::message(link_1.clone()), 0);
    expected_behaviour.insert(LogEntry::message(link_2.clone()), 0);
    expected_behaviour.insert(LogEntry::message(link_3.clone()), 4);
    expected_behaviour.insert(LogEntry::message(link_4.clone()), 4);
    expected_behaviour.insert(LogEntry::message(link_5.clone()), 4);
    expected_behaviour.insert(LogEntry::message(router_0.clone()), 0);
    expected_behaviour.insert(LogEntry::message(router_1.clone()), 8);
    expected_behaviour.insert(LogEntry::found_response_upstream(echo_protocol_0.clone()), 0);
    expected_behaviour.insert(LogEntry::found_response_upstream(echo_protocol_1.clone()), 0);
    expected_behaviour.insert(LogEntry::response_arrived_downstream(echo_protocol_0.clone()), 0);
    expected_behaviour.insert(LogEntry::response_arrived_downstream(echo_protocol_1.clone()), 0);
    expected_behaviour.insert(LogEntry::forward_response_downstream(router_0.clone()), 0);
    expected_behaviour.insert(LogEntry::forward_response_downstream(router_1.clone()), 0);
    expected_behaviour.insert(LogEntry::forward_request_upstream(router_0.clone()), 0);
    expected_behaviour.insert(LogEntry::forward_request_upstream(router_1.clone()), 4);
*/
    link3.corrupt(Corruption::Order);
    expected_behaviour.insert(LogEntry::message(echo_protocol_0.clone()), 10);
    expected_behaviour.insert(LogEntry::message(echo_protocol_1.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_0.clone()), 10);
    expected_behaviour.insert(LogEntry::message(link_1.clone()), 10);
    expected_behaviour.insert(LogEntry::message(link_2.clone()), 10);
    expected_behaviour.insert(LogEntry::message(link_3.clone()), 10);
    expected_behaviour.insert(LogEntry::message(link_4.clone()), 9);
    expected_behaviour.insert(LogEntry::message(link_5.clone()), 9);
    expected_behaviour.insert(LogEntry::message(router_0.clone()), 20);
    expected_behaviour.insert(LogEntry::message(router_1.clone()), 18);
    expected_behaviour.insert(LogEntry::found_response_upstream(echo_protocol_0.clone()), 5);
    expected_behaviour.insert(LogEntry::found_response_upstream(echo_protocol_1.clone()), 0);
    expected_behaviour.insert(LogEntry::response_arrived_downstream(echo_protocol_0.clone()), 0);
    expected_behaviour.insert(LogEntry::response_arrived_downstream(echo_protocol_1.clone()), 4);
    expected_behaviour.insert(LogEntry::forward_response_downstream(router_0.clone()), 5);
    expected_behaviour.insert(LogEntry::forward_response_downstream(router_1.clone()), 5);
    expected_behaviour.insert(LogEntry::forward_request_upstream(router_0.clone()), 5);
    expected_behaviour.insert(LogEntry::forward_request_upstream(router_1.clone()), 4);
/*
    link3.corrupt(Corruption::Presence);
    expected_behaviour.insert(LogEntry::message(echo_protocol_0.clone()), 6);
    expected_behaviour.insert(LogEntry::message(echo_protocol_1.clone()), 7);
    expected_behaviour.insert(LogEntry::message(link_0.clone()), 6);
    expected_behaviour.insert(LogEntry::message(link_1.clone()), 6);
    expected_behaviour.insert(LogEntry::message(link_2.clone()), 6);
    expected_behaviour.insert(LogEntry::message(link_3.clone()), 6);
    expected_behaviour.insert(LogEntry::message(link_4.clone()), 7);
    expected_behaviour.insert(LogEntry::message(link_5.clone()), 7);
    expected_behaviour.insert(LogEntry::message(router_0.clone()), 12);
    expected_behaviour.insert(LogEntry::message(router_1.clone()), 14);
    expected_behaviour.insert(LogEntry::found_response_upstream(echo_protocol_0.clone()), 3);
    expected_behaviour.insert(LogEntry::found_response_upstream(echo_protocol_1.clone()), 0);
    expected_behaviour.insert(LogEntry::response_arrived_downstream(echo_protocol_0.clone()), 0);
    expected_behaviour.insert(LogEntry::response_arrived_downstream(echo_protocol_1.clone()), 3);
    expected_behaviour.insert(LogEntry::forward_response_downstream(router_0.clone()), 3);
    expected_behaviour.insert(LogEntry::forward_response_downstream(router_1.clone()), 3);
    expected_behaviour.insert(LogEntry::forward_request_upstream(router_0.clone()), 3);
    expected_behaviour.insert(LogEntry::forward_request_upstream(router_1.clone()), 4);
*/
    echo_protocol0.run()?;
    link0.run()?;
    link1.run()?;
    broker0.run()?;
    link2.run()?;
    link3.run()?;
    broker1.run()?;
    link4.run()?;
    link5.run()?;
    echo_protocol1.run()?;
    let response = std::thread::spawn(move || {
        let data: String = echo_protocol1.unreliable_sequenced_cleartext_ping(echo_protocol_sid0.public_id()).unwrap();
        actual_behaviour.end();
        data
    });
    process_network(expected_behaviour, receiver)?;
    let actual_response = response.join().expect("failed to extract data from JoinHandle");
    let expected_response = "pong".to_string();
    if actual_response != expected_response{
        Err(anyhow!("actual returned data (1st under) didn't match expected returned data (2nd under):\n{}\n{}", actual_response, expected_response))
    } else {
        Ok(())
    }
}
/*
    debug!("unreliable unordered cleartext ping");
    let pong: String = echo_protocol1.unreliable_unordered_cleartext_ping(echo_protocol_sid0.public_id())?;
    debug!("unreliable unordered cleartext {:?}", pong);

    debug!("unreliable unordered cyphertext ping");
    let pong: String = echo_protocol0.unreliable_unordered_cyphertext_ping(echo_protocol_sid1.public_id())?;
    debug!("unreliable unordered cyphertext {:?}", pong);

    debug!("unreliable sequenced cleartext ping");
    let pong: String = echo_protocol1.unreliable_sequenced_cleartext_ping(echo_protocol_sid0.public_id())?;
    debug!("unreliable sequenced cleartext {:?}", pong);

    debug!("unreliable sequenced cyphertext ping");
    let pong: String = echo_protocol0.unreliable_sequenced_cyphertext_ping(echo_protocol_sid1.public_id())?;
    debug!("unreliable sequenced cyphertext {:?}", pong);

    debug!("reliable unordered cleartext ping");
    let pong: String = echo_protocol1.reliable_unordered_cleartext_ping(echo_protocol_sid0.public_id())?;
    debug!("reliable unordered cleartext {:?}", pong);

    debug!("reliable unordered cyphertext ping");
    let pong: String = echo_protocol0.reliable_unordered_cyphertext_ping(echo_protocol_sid1.public_id())?;
    debug!("reliable unordered cyphertext {:?}", pong);

    debug!("reliable ordered cleartext ping");
    let pong: String = echo_protocol1.reliable_ordered_cleartext_ping(echo_protocol_sid0.public_id())?;
    debug!("reliable ordered cleartext {:?}", pong);

    debug!("reliable ordered cyphertext ping");
    let pong: String = echo_protocol0.reliable_ordered_cyphertext_ping(echo_protocol_sid1.public_id())?;
    debug!("reliable ordered cyphertext {:?}", pong);

    debug!("reliable sequenced cleartext ping");
    let pong: String = echo_protocol1.reliable_sequenced_cleartext_ping(echo_protocol_sid0.public_id())?;
    debug!("reliable sequenced cleartext {:?}", pong);

    debug!("reliable sequenced cyphertext ping");
    let pong: String = echo_protocol0.reliable_sequenced_cyphertext_ping(echo_protocol_sid1.public_id())?;
    debug!("reliable sequenced cyphertext {:?}", pong);
*/
