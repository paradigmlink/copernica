use {
    crossbeam_channel::{Sender},
    std::fmt::{self},
};
#[derive(Clone, Debug)]
pub enum Operations {
    On { tx: Sender<LogEntry> },
    Off,
}
impl Operations {
    pub fn turned_on(tx: Sender<LogEntry>) -> Self {
        Operations::On { tx }
    }
    pub fn turned_off() -> Self {
        Operations::Off
    }
    // convenience function to reduce typing on Protocol, Link, Router API
    pub fn label(&self, label: &str) -> (String, Self) {
        (label.to_string(), self.clone())
    }
    pub fn end(&self) {
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::End) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn register_protocol(&self, label: String) {
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::register(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn register_link(&self, label: String) {
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::register(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn register_router(&self, label: String) {
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::register(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn message_from(&self, label: String) {
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::message(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn found_response_upstream(&self, label: String) {
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::found_response_upstream(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn response_arrived_downstream(&self, label: String) {
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::response_arrived_downstream(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn forward_response_downstream(&self, label: String) {
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::forward_response_downstream(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn forward_request_upstream(&self, label: String) {
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::forward_request_upstream(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum LogEntry {
    End,
    Register {
        label: String,
    },
    Message {
        label: String,
    },
    FoundResponseUpstream {
        label: String,
    },
    ResponseArrivedDownstream {
        label: String,
    },
    ForwardResponseDownstream {
        label: String,
    },
    ForwardRequestUpstream {
        label: String,
    },
}
impl LogEntry {
    pub fn end() -> Self {
        LogEntry::End
    }
    pub fn register(label: &str) -> Self {
        //LogEntry::Register { label: format!("register\t\t\t| {}", label) }
        LogEntry::Register { label: format!("expected_behaviour.insert(LogEntry::register({}.clone()),", label) }
    }
    pub fn message(label: &str) -> Self {
        //LogEntry::Message { label: format!("message\t\t\t\t| {}", &label)  }
        LogEntry::Message { label: format!("expected_behaviour.insert(LogEntry::message({}.clone()),", &label)  }
    }
    pub fn found_response_upstream(label: &str) -> Self {
        //LogEntry::FoundResponseUpstream { label: format!("found_response_upstream\t\t| {}", &label)  }
        LogEntry::FoundResponseUpstream { label: format!("expected_behaviour.insert(LogEntry::found_response_upstream({}.clone()),", &label)  }
    }
    pub fn response_arrived_downstream(label: &str) -> Self {
        //LogEntry::ResponseArrivedDownstream { label: format!("response_arrived_downstream\t| {}", &label)  }
        LogEntry::ResponseArrivedDownstream { label: format!("expected_behaviour.insert(LogEntry::response_arrived_downstream({}.clone()),", &label)  }
    }
    pub fn forward_request_upstream(label: &str) -> Self {
        //LogEntry::ForwardRequestUpstream { label: format!("forward_request_upstream\t| {}", &label)  }
        LogEntry::ForwardRequestUpstream { label: format!("expected_behaviour.insert(LogEntry::forward_request_upstream({}.clone()),", &label)  }
    }
    pub fn forward_response_downstream(label: &str) -> Self {
        //LogEntry::ForwardResponseDownstream { label: format!("forward_response_downstream\t| {}", &label)  }
        LogEntry::ForwardResponseDownstream { label: format!("expected_behaviour.insert(LogEntry::forward_response_downstream({}.clone()),", &label)  }
    }
}
impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            LogEntry::Register { label } => {
                format!("{}", label)
            }
            LogEntry::Message { label } => {
                format!("{}", label)
            },
            LogEntry::FoundResponseUpstream { label } => {
                format!("{}", label)
            },
            LogEntry::ResponseArrivedDownstream { label } => {
                format!("{}", label)
            },
            LogEntry::ForwardResponseDownstream { label } => {
                format!("{}", label)
            },
            LogEntry::ForwardRequestUpstream { label } => {
                format!("{}", label)
            },
            LogEntry::End => {
                format!("end")
            },
        };
        write!(f, "{}", out)
    }
}
