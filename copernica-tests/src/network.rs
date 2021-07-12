use {
    anyhow::{Result, anyhow},
    copernica_protocols::{Echo, Protocol},
    copernica_common::{LinkId, ReplyTo, PrivateIdentityInterface, PublicIdentityInterface, Operations, LogEntry},
    copernica_broker::{Broker},
    copernica_links::{Link, MpscChannel, MpscCorruptor, UdpIp},
    scaffolding::{ group, single, Ordering, TestTree},
    std::sync::mpsc::{channel},
    std::collections::HashMap,
};
pub fn network_echo(ordering: Ordering) -> TestTree {
    group!(
        format!("Unit tests, ordering with {:?}", ordering),
        ordering,
        [
            single!(|| { ping_pong() }),
        ]
    )
}
pub fn ping_pong() -> Result<()> {
    let (sender, receiver) = channel::<LogEntry>();
    let actual_behaviour = Operations::turned_on(sender);
    let mut broker0 = Broker::new(actual_behaviour.label("router0"));
    let mut broker1 = Broker::new(actual_behaviour.label("router1"));
    let echo_protocol_sid0 = PrivateIdentityInterface::new_key();
    let echo_protocol_sid1 = PrivateIdentityInterface::new_key();
    let mut echo_protocol0: Echo = Protocol::new(echo_protocol_sid0.clone(), actual_behaviour.label("echo_protocol0"));
    let mut echo_protocol1: Echo = Protocol::new(echo_protocol_sid1.clone(), actual_behaviour.label("echo_protocol1"));
    // echo_protocol0 to broker0
    let link_sid0 = PrivateIdentityInterface::new_key();
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link_id0 = LinkId::link_with_type(link_sid0.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let link_id1 = LinkId::link_with_type(link_sid1.clone(), PublicIdentityInterface::Absent, ReplyTo::Mpsc);
    let mut link0: MpscChannel = Link::new(link_id0.clone(), actual_behaviour.label("link0"), broker0.peer_with_link(link_id0.clone())?)?;
    let mut link1: MpscChannel = Link::new(link_id1.clone(), actual_behaviour.label("link1"), echo_protocol0.peer_with_link(link_id0.clone())?)?;
    link0.female(link1.male());
    link1.female(link0.male());
    // broker0 to broker1
    let link_sid2 = PrivateIdentityInterface::new_key();
    let link_sid3 = PrivateIdentityInterface::new_key();
    let link_id2 = LinkId::link_with_type(link_sid2.clone(), PublicIdentityInterface::new(link_sid3.public_id()), ReplyTo::Mpsc);
    let link_id3 = LinkId::link_with_type(link_sid3.clone(), PublicIdentityInterface::new(link_sid2.public_id()), ReplyTo::Mpsc);
    let mut link2: MpscCorruptor = Link::new(link_id2.clone(), actual_behaviour.label("link2"), broker0.peer_with_link(link_id2.clone())?)?;
    let mut link3: MpscCorruptor = Link::new(link_id3.clone(), actual_behaviour.label("link3"), broker1.peer_with_link(link_id3.clone())?)?;
    link2.female(link3.male());
    link3.female(link2.male());
    // broker1 to echo_protocol1
    let link_sid4 = PrivateIdentityInterface::new_key();
    let link_sid5 = PrivateIdentityInterface::new_key();
    let address4 = ReplyTo::UdpIp("127.0.0.1:50002".parse()?);
    let address5 = ReplyTo::UdpIp("127.0.0.1:50003".parse()?);
    let link_id4 = LinkId::link_with_type(link_sid4.clone(), PublicIdentityInterface::new(link_sid5.public_id()), address4.clone());
    let link_id5 = LinkId::link_with_type(link_sid5.clone(), PublicIdentityInterface::new(link_sid4.public_id()), address5.clone());
    let mut link4: UdpIp = Link::new(link_id4.clone(), actual_behaviour.label("link4"), broker1.peer_with_link(link_id4.remote(address5)?)?)?;
    let mut link5: UdpIp = Link::new(link_id5.clone(), actual_behaviour.label("link5"), echo_protocol1.peer_with_link(link_id5.remote(address4)?)?)?;
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
    let echo_protocol_sid0_in_thread = echo_protocol_sid0.clone();
    let data = std::thread::spawn(move || {
        let data: String = echo_protocol1.unreliable_unordered_cleartext_ping(echo_protocol_sid0_in_thread.clone().public_id()).unwrap();
        actual_behaviour.end();
        data
    });
    let mut expected_behaviour: HashMap<LogEntry, i32> = HashMap::new();
    expected_behaviour.insert(LogEntry::register("router0"), 1);
    expected_behaviour.insert(LogEntry::register("router1"), 1);
    expected_behaviour.insert(LogEntry::register("echo_protocol0"), 1);
    expected_behaviour.insert(LogEntry::register("echo_protocol1"), 1);
    expected_behaviour.insert(LogEntry::register("link0"), 1);
    expected_behaviour.insert(LogEntry::register("link1"), 1);
    expected_behaviour.insert(LogEntry::register("link2"), 1);
    expected_behaviour.insert(LogEntry::register("link3"), 1);
    expected_behaviour.insert(LogEntry::register("link4"), 1);
    expected_behaviour.insert(LogEntry::register("link5"), 1);
    expected_behaviour.insert(LogEntry::message("echo_protocol0"), 0);
    expected_behaviour.insert(LogEntry::message("echo_protocol1"), 4);
    expected_behaviour.insert(LogEntry::message("link0"), 8);
    expected_behaviour.insert(LogEntry::message("link1"), 8);
    expected_behaviour.insert(LogEntry::message("link2"), 8);
    expected_behaviour.insert(LogEntry::message("link3"), 8);
    expected_behaviour.insert(LogEntry::message("link4"), 8);
    expected_behaviour.insert(LogEntry::message("link5"), 8);
    expected_behaviour.insert(LogEntry::found_response("router0"), 0);
    expected_behaviour.insert(LogEntry::found_response("router1"), 0);
    expected_behaviour.insert(LogEntry::forward_response_downstream("router0"), 4);
    expected_behaviour.insert(LogEntry::forward_response_downstream("router1"), 4);
    expected_behaviour.insert(LogEntry::forward_request_upstream("router0"), 4);
    expected_behaviour.insert(LogEntry::forward_request_upstream("router1"), 4);
    loop {
        let log_entry = receiver.recv()?;
        match log_entry {
            LogEntry::Register { ref label } => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::Message { ref label } => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::FoundResponse { ref label } => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::ForwardResponseDownstream { ref label } => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::ForwardRequestUpstream { ref label } => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::End => {
                for (key, value) in &expected_behaviour {
                    if value != &0 {
                        return Err(anyhow!("Node \"{}\" has an unexpected amount of messages sent: {}", key, value))
                    }
                }
                break;
            },
        }
    }
    let actual_data = data.join().expect("failed to extract data from JoinHandle");
    let expected_data = "pong".to_string();
    if actual_data != expected_data {
        Err(anyhow!("actual returned data (1st under) didn't match expected returned data (2nd under):\n{}\n{}", actual_data, expected_data))
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
