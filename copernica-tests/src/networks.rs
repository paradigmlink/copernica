use {
    anyhow::{Result, anyhow},
    copernica_protocols::{Echo, Protocol},
    copernica_packets::{LinkId, ReplyTo, PrivateIdentityInterface, PublicIdentityInterface},
    copernica_common::{Operations, LogEntry},
    copernica_broker::{Broker},
    copernica_links::{Link, MpscChannel, MpscCorruptor, UdpIpV4, Corruption},
    crate::process_network,
    scaffolding::{ group, single, Ordering, TestTree, setting, settings::Timeout},
    crossbeam_channel::{unbounded},
    std::{
        time::Duration,
        collections::HashMap,
    },
};
pub fn basic_networks(ordering: Ordering) -> TestTree {
    group!(
        format!("Unit tests, ordering with {:?}", ordering),
        ordering,
        [
            setting!(Timeout(Duration::from_secs(5))),
            single!(|| { three_protocol_one_broker() }),
            single!(|| { two_protocol_two_broker_one_protocol() }),
        ]
    )
}
pub fn three_protocol_one_broker() -> Result<()> {
    let router_0 = "router_0";
    let echo_protocol_0 = "echo_protocol_0";
    let echo_protocol_1 = "echo_protocol_1";
    let echo_protocol_2 = "echo_protocol_2";
    let link_0 = "link_0";
    let link_1 = "link_1";
    let link_2 = "link_2";
    let link_3 = "link_3";
    let link_4 = "link_4";
    let link_5 = "link_5";
    let (sender, receiver) = unbounded::<LogEntry>();
    let actual_behaviour = Operations::turned_on(sender);
    let mut broker0 = Broker::new(actual_behaviour.label(router_0.clone()));
    let echo_protocol_sid0 = PrivateIdentityInterface::new_key();
    let echo_protocol_sid1 = PrivateIdentityInterface::new_key();
    let echo_protocol_sid2 = PrivateIdentityInterface::new_key();
    let mut echo_protocol0: Echo = Protocol::new(echo_protocol_sid0.clone(), actual_behaviour.label(echo_protocol_0.clone()));
    let mut echo_protocol1: Echo = Protocol::new(echo_protocol_sid1.clone(), actual_behaviour.label(echo_protocol_1.clone()));
    let mut echo_protocol2: Echo = Protocol::new(echo_protocol_sid2.clone(), actual_behaviour.label(echo_protocol_2.clone()));
    let link_sid0 = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id0 = LinkId::link_with_type(link_sid0.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let mut link0: MpscChannel = Link::new(link_id0.clone(), actual_behaviour.label(link_0.clone()), broker0.peer_with_link(link_id0.clone())?)?;
    let mut link1: MpscChannel = Link::new(link_id1.clone(), actual_behaviour.label(link_1.clone()), echo_protocol0.peer_with_link(link_id1.clone())?)?;
    link0.female(link1.male());
    link1.female(link0.male());
    let link_sid2 = PrivateIdentityInterface::new_key();
    let link_sid3 = PrivateIdentityInterface::new_key();
    let link_id2 = LinkId::link_with_type(link_sid2.clone(), PublicIdentityInterface::new(link_sid3.public_id()), ReplyTo::Mpsc);
    let link_id3 = LinkId::link_with_type(link_sid3.clone(), PublicIdentityInterface::new(link_sid2.public_id()), ReplyTo::Mpsc);
    let mut link2: MpscCorruptor = Link::new(link_id2.clone(), actual_behaviour.label(link_2.clone()), broker0.peer_with_link(link_id2.clone())?)?;
    let mut link3: MpscCorruptor = Link::new(link_id3.clone(), actual_behaviour.label(link_3.clone()), echo_protocol1.peer_with_link(link_id3.clone())?)?;
    link2.female(link3.male());
    link3.female(link2.male());
    let link_sid4 = PrivateIdentityInterface::new_key();
    let link_sid5 = PrivateIdentityInterface::new_key();
    let address4 = ReplyTo::UdpIpV4("127.0.0.1:50049".parse()?);
    let address5 = ReplyTo::UdpIpV4("127.0.0.1:50050".parse()?);
    let link_id4 = LinkId::link_with_type(link_sid4.clone(), PublicIdentityInterface::new(link_sid5.public_id()), address4.clone());
    let link_id5 = LinkId::link_with_type(link_sid5.clone(), PublicIdentityInterface::new(link_sid4.public_id()), address5.clone());
    let mut link4: UdpIpV4 = Link::new(link_id4.clone(), actual_behaviour.label(link_4.clone()), broker0.peer_with_link(link_id4.remote(address5)?)?)?;
    let mut link5: UdpIpV4 = Link::new(link_id5.clone(), actual_behaviour.label(link_5.clone()), echo_protocol2.peer_with_link(link_id5.remote(address4)?)?)?;
    let mut expected_behaviour: HashMap<LogEntry, i32> = HashMap::new();
    expected_behaviour.insert(LogEntry::register(router_0.clone()), 1);
    expected_behaviour.insert(LogEntry::register(echo_protocol_0.clone()), 1);
    expected_behaviour.insert(LogEntry::register(echo_protocol_1.clone()), 1);
    expected_behaviour.insert(LogEntry::register(echo_protocol_2.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_0.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_1.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_2.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_3.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_4.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_5.clone()), 1);
    link3.corrupt(Corruption::Immune);

    expected_behaviour.insert(LogEntry::response_arrived_downstream(echo_protocol_2.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_0.clone()), 16);
    expected_behaviour.insert(LogEntry::message(link_2.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_3.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_1.clone()), 16);
    expected_behaviour.insert(LogEntry::message(link_5.clone()), 16);
    expected_behaviour.insert(LogEntry::message(echo_protocol_0.clone()), 16);
    expected_behaviour.insert(LogEntry::forward_response_downstream(router_0.clone()), 8);
    expected_behaviour.insert(LogEntry::message(echo_protocol_2.clone()), 16);
    expected_behaviour.insert(LogEntry::message(echo_protocol_1.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_4.clone()), 16);
    expected_behaviour.insert(LogEntry::found_response_upstream(echo_protocol_0.clone()), 8);
    expected_behaviour.insert(LogEntry::response_arrived_downstream(echo_protocol_1.clone()), 0);
    expected_behaviour.insert(LogEntry::forward_request_upstream(router_0.clone()), 8);
    expected_behaviour.insert(LogEntry::message(router_0.clone()), 40);

    echo_protocol0.run()?;
    link0.run()?;
    link1.run()?;
    broker0.run()?;
    link2.run()?;
    link3.run()?;
    echo_protocol1.run()?;
    link4.run()?;
    link5.run()?;
    echo_protocol2.run()?;
    let response = std::thread::spawn(move || {
        let data: String = echo_protocol2.unreliable_sequenced_cleartext_ping(echo_protocol_sid0.public_id()).unwrap();
        actual_behaviour.end();
        data
    });
    process_network(expected_behaviour, receiver)?;
    let actual_response = response.join().expect("failed to extract data from JoinHandle");
    let expected_response = "pingpong".to_string();
    if actual_response != expected_response{
        Err(anyhow!("actual returned data (1st under) didn't match expected returned data (2nd under):\n{}\n{}", actual_response, expected_response))
    } else {
        Ok(())
    }
}
pub fn two_protocol_two_broker_one_protocol() -> Result<()> {
    let router_0 = "router_0";
    let router_1 = "router_1";
    let echo_protocol_0 = "echo_protocol_0";
    let echo_protocol_1 = "echo_protocol_1";
    let echo_protocol_2 = "echo_protocol_2";
    let link_0 = "link_0";
    let link_1 = "link_1";
    let link_2 = "link_2";
    let link_3 = "link_3";
    let link_4 = "link_4";
    let link_5 = "link_5";
    let link_6 = "link_6";
    let link_7 = "link_7";
    let (sender, receiver) = unbounded::<LogEntry>();
    let actual_behaviour = Operations::turned_on(sender);
    let mut broker0 = Broker::new(actual_behaviour.label(router_0.clone()));
    let mut broker1 = Broker::new(actual_behaviour.label(router_1.clone()));
    let echo_protocol_sid0 = PrivateIdentityInterface::new_key();
    let echo_protocol_sid1 = PrivateIdentityInterface::new_key();
    let echo_protocol_sid2 = PrivateIdentityInterface::new_key();
    let mut echo_protocol0: Echo = Protocol::new(echo_protocol_sid0.clone(), actual_behaviour.label(echo_protocol_0.clone()));
    let mut echo_protocol1: Echo = Protocol::new(echo_protocol_sid1.clone(), actual_behaviour.label(echo_protocol_1.clone()));
    let mut echo_protocol2: Echo = Protocol::new(echo_protocol_sid2.clone(), actual_behaviour.label(echo_protocol_2.clone()));
    let link_sid0 = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id0 = LinkId::link_with_type(link_sid0.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let mut link0: MpscChannel = Link::new(link_id0.clone(), actual_behaviour.label(link_0.clone()), broker0.peer_with_link(link_id0.clone())?)?;
    let mut link1: MpscChannel = Link::new(link_id1.clone(), actual_behaviour.label(link_1.clone()), echo_protocol0.peer_with_link(link_id1.clone())?)?;
    link0.female(link1.male());
    link1.female(link0.male());
    let link_sid2 = PrivateIdentityInterface::new_key();
    let link_sid3 = PrivateIdentityInterface::new_key();
    let link_id2 = LinkId::link_with_type(link_sid2.clone(), PublicIdentityInterface::new(link_sid3.public_id()), ReplyTo::Mpsc);
    let link_id3 = LinkId::link_with_type(link_sid3.clone(), PublicIdentityInterface::new(link_sid2.public_id()), ReplyTo::Mpsc);
    let mut link2: MpscCorruptor = Link::new(link_id2.clone(), actual_behaviour.label(link_2.clone()), broker0.peer_with_link(link_id2.clone())?)?;
    let mut link3: MpscCorruptor = Link::new(link_id3.clone(), actual_behaviour.label(link_3.clone()), echo_protocol1.peer_with_link(link_id3.clone())?)?;
    link2.female(link3.male());
    link3.female(link2.male());
    let link_sid4 = PrivateIdentityInterface::new_key();
    let link_sid5 = PrivateIdentityInterface::new_key();
    let link_id4 = LinkId::link_with_type(link_sid4.clone(), PublicIdentityInterface::new(link_sid5.public_id()), ReplyTo::Mpsc);
    let link_id5 = LinkId::link_with_type(link_sid5.clone(), PublicIdentityInterface::new(link_sid4.public_id()), ReplyTo::Mpsc);
    let mut link4: MpscCorruptor = Link::new(link_id4.clone(), actual_behaviour.label(link_4.clone()), broker0.peer_with_link(link_id4.clone())?)?;
    let mut link5: MpscCorruptor = Link::new(link_id5.clone(), actual_behaviour.label(link_5.clone()), broker1.peer_with_link(link_id5.clone())?)?;
    link4.female(link5.male());
    link5.female(link4.male());
    let link_sid6 = PrivateIdentityInterface::new_key();
    let link_sid7 = PrivateIdentityInterface::new_key();
    let address6 = ReplyTo::UdpIpV4("127.0.0.1:50051".parse()?);
    let address7 = ReplyTo::UdpIpV4("127.0.0.1:50052".parse()?);
    let link_id6 = LinkId::link_with_type(link_sid6.clone(), PublicIdentityInterface::new(link_sid7.public_id()), address6.clone());
    let link_id7 = LinkId::link_with_type(link_sid7.clone(), PublicIdentityInterface::new(link_sid6.public_id()), address7.clone());
    let mut link6: UdpIpV4 = Link::new(link_id6.clone(), actual_behaviour.label(link_6.clone()), broker1.peer_with_link(link_id6.remote(address7)?)?)?;
    let mut link7: UdpIpV4 = Link::new(link_id7.clone(), actual_behaviour.label(link_7.clone()), echo_protocol2.peer_with_link(link_id7.remote(address6)?)?)?;
    let mut expected_behaviour: HashMap<LogEntry, i32> = HashMap::new();
    expected_behaviour.insert(LogEntry::register(router_0.clone()), 1);
    expected_behaviour.insert(LogEntry::register(router_1.clone()), 1);
    expected_behaviour.insert(LogEntry::register(echo_protocol_0.clone()), 1);
    expected_behaviour.insert(LogEntry::register(echo_protocol_1.clone()), 1);
    expected_behaviour.insert(LogEntry::register(echo_protocol_2.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_0.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_1.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_2.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_3.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_4.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_5.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_6.clone()), 1);
    expected_behaviour.insert(LogEntry::register(link_7.clone()), 1);
    link3.corrupt(Corruption::Immune);

    expected_behaviour.insert(LogEntry::message(link_7.clone()), 16);
    expected_behaviour.insert(LogEntry::found_response_upstream(echo_protocol_0.clone()), 8);
    expected_behaviour.insert(LogEntry::forward_request_upstream(router_1.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_3.clone()), 8);
    expected_behaviour.insert(LogEntry::message(router_1.clone()), 32);
    expected_behaviour.insert(LogEntry::message(link_1.clone()), 16);
    expected_behaviour.insert(LogEntry::message(link_4.clone()), 16);
    expected_behaviour.insert(LogEntry::forward_response_downstream(router_1.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_5.clone()), 16);
    expected_behaviour.insert(LogEntry::response_arrived_downstream(echo_protocol_2.clone()), 8);
    expected_behaviour.insert(LogEntry::message(router_0.clone()), 40);
    expected_behaviour.insert(LogEntry::forward_request_upstream(router_0.clone()), 8);
    expected_behaviour.insert(LogEntry::message(echo_protocol_1.clone()), 8);
    expected_behaviour.insert(LogEntry::message(link_2.clone()), 8);
    expected_behaviour.insert(LogEntry::message(echo_protocol_2.clone()), 16);
    expected_behaviour.insert(LogEntry::forward_response_downstream(router_0.clone()), 8);
    expected_behaviour.insert(LogEntry::message(echo_protocol_0.clone()), 16);
    expected_behaviour.insert(LogEntry::message(link_6.clone()), 16);
    expected_behaviour.insert(LogEntry::message(link_0.clone()), 16);

    echo_protocol0.run()?;
    link0.run()?;
    link1.run()?;
    broker0.run()?;
    broker1.run()?;
    link2.run()?;
    link3.run()?;
    echo_protocol1.run()?;
    link4.run()?;
    link5.run()?;
    link6.run()?;
    link7.run()?;
    echo_protocol2.run()?;
    let response = std::thread::spawn(move || {
        let data: String = echo_protocol2.unreliable_sequenced_cleartext_ping(echo_protocol_sid0.public_id()).unwrap();
        actual_behaviour.end();
        data
    });
    process_network(expected_behaviour, receiver)?;
    let actual_response = response.join().expect("failed to extract data from JoinHandle");
    let expected_response = "pingpong".to_string();
    if actual_response != expected_response{
        Err(anyhow!("actual returned data (1st under) didn't match expected returned data (2nd under):\n{}\n{}", actual_response, expected_response))
    } else {
        Ok(())
    }
}
