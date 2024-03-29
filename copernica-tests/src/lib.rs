extern crate fern;
extern crate chrono;
mod common;
mod unreliable_sequenced_cyphertext;
mod reliable_sequenced_cyphertext;
mod reliable_ordered_cyphertext;
mod unreliable_sequenced_cleartext;
mod reliable_sequenced_cleartext;
mod reliable_ordered_cleartext;
mod primitive_link_packet;
mod networks;
mod logger;
pub use {
    unreliable_sequenced_cleartext::{unreliable_sequenced_cleartext_ping_pong},
    unreliable_sequenced_cyphertext::{unreliable_sequenced_cyphertext_ping_pong},
    reliable_sequenced_cleartext::{reliable_sequenced_cleartext_ping_pong},
    reliable_sequenced_cyphertext::{reliable_sequenced_cyphertext_ping_pong},
    reliable_ordered_cleartext::{reliable_ordered_cleartext_ping_pong},
    reliable_ordered_cyphertext::{reliable_ordered_cyphertext_ping_pong},
    primitive_link_packet::{primitive_link_packet},
    networks::{basic_networks, },
    logger::{setup_logging},
};
use {
    anyhow::{Result, anyhow},
    copernica_common::{LogEntry},
    crossbeam_channel::{Receiver},
    std::collections::HashMap,
};
pub fn process_network(mut expected_behaviour: HashMap<LogEntry, i32>, receiver: Receiver<LogEntry>) -> Result<()> {
    let ref_expected_behaviour = expected_behaviour.clone();
    let mut error: String = "Corrections below:\n".into();
    loop {
        let log_entry = receiver.recv()?;
        match log_entry {
            LogEntry::Register(label) => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::Message(label) => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::FoundResponseUpstream(label) => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::ResponseArrivedDownstream(label) => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::ForwardResponseDownstream(label) => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::ForwardRequestUpstream(label) => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::End => {
                for (key, value) in &expected_behaviour {
                    if value != &0 {
                        if let Some(ref_value) = ref_expected_behaviour.get(key) {
                            error.push_str(&format!("{} {});\n", key, ref_value - value))
                        }
                    }
                }
                if error == "Corrections below:\n".to_string() {
                    break
                } else {
                    return Err(anyhow!("{}", error))
                }
            },
        }
    }
    Ok(())
}
