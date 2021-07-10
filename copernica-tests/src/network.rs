use {
    anyhow::{Result, anyhow},
    copernica_protocols::{Echo, Protocol},
    copernica_common::{LinkId, ReplyTo, PrivateIdentityInterface, PublicIdentityInterface, Operations, LogEntry},
    copernica_broker::{Broker},
    copernica_links::{Link, MpscChannel, MpscCorruptor, UdpIp},
    scaffolding::{ group, single, Ordering, TestTree},
    std::sync::mpsc::{channel},
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
    actual_behaviour.start();
    let mut broker0 = Broker::new(actual_behaviour.clone());
    let mut broker1 = Broker::new(actual_behaviour.clone());
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
    let mut expected_behaviour: Vec<LogEntry> = vec![
        LogEntry::Start,
        LogEntry::protocol(echo_protocol_sid0.clone().public_id(), "echo_protocol0".to_string(), "".to_string()),
        LogEntry::protocol(echo_protocol_sid1.clone().public_id(), "echo_protocol1".to_string(), "".to_string()),
        LogEntry::router(broker0.id(), "".to_string()),
        LogEntry::link(link_sid0.public_id(), "link0".to_string(), "".to_string()),
        LogEntry::link(link_sid1.public_id(), "link1".to_string(), "".to_string()),
        LogEntry::router(broker0.id(), "".to_string()),
        LogEntry::link(link_sid2.public_id(), "link2".to_string(), "".to_string()),
        LogEntry::router(broker1.id(), "".to_string()),
        LogEntry::link(link_sid3.public_id(), "link3".to_string(), "".to_string()),
        LogEntry::router(broker1.id(), "".to_string()),
        LogEntry::link(link_sid4.public_id(), "link4".to_string(), "".to_string()),
        LogEntry::link(link_sid5.public_id(), "link5".to_string(), "".to_string()),
        LogEntry::pid_to_pid(link_sid5.public_id(), PublicIdentityInterface::Present { public_identity: link_sid4.public_id() }, "".to_string(), "".to_string()),
        LogEntry::pid_to_pid(link_sid5.public_id(), PublicIdentityInterface::Present { public_identity: link_sid4.public_id() }, "".to_string(), "".to_string()),
        LogEntry::pid_to_pid(link_sid5.public_id(), PublicIdentityInterface::Present { public_identity: link_sid4.public_id() }, "".to_string(), "".to_string()),
        LogEntry::pid_to_pid(link_sid5.public_id(), PublicIdentityInterface::Present { public_identity: link_sid4.public_id() }, "".to_string(), "".to_string()),
        LogEntry::End,
    ];
    expected_behaviour.reverse();
    loop {
        if expected_behaviour.len() == 0 { break }
        let expected_log_entry = expected_behaviour.pop().unwrap();
        let actual_log_entry = receiver.recv()?;
        if actual_log_entry != expected_log_entry {
            return Err(anyhow!("actual_log_entry (1st under) didn't match expected_log_entry (2nd under)\n{}\n{}", actual_log_entry, expected_log_entry));
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
