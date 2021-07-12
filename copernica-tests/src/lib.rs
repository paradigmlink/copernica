mod common;
mod network;
pub use {
    network::network_echo,
};
use {
    anyhow::{Result, anyhow},
    copernica_common::{LogEntry},
    std::sync::mpsc::{Receiver},
    std::collections::HashMap,
};
pub fn process_network(mut expected_behaviour: HashMap<LogEntry, i32>, receiver: Receiver<LogEntry>) -> Result<()> {
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
            LogEntry::FoundResponseUpstream { ref label } => {
                if let Some(count) = expected_behaviour.get_mut(&log_entry) {
                    *count -= 1;
                } else {
                    return Err(anyhow!("\"{}\" not present in expected_behaviour", label))
                }
            },
            LogEntry::ResponseArrivedDownstream { ref label } => {
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
    Ok(())
}
