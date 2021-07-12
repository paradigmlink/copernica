use {
    std::{
        sync::mpsc::Sender,
        fmt::{self},
    },
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
        LogEntry::Register { label: format!("registering node: {}", label) }
    }
    pub fn message(label: &str) -> Self {
        LogEntry::Message { label: format!("message sent from node: {}", &label)  }
    }
    pub fn found_response_upstream(label: &str) -> Self {
        LogEntry::FoundResponseUpstream { label: format!("found response in node: {}", &label)  }
    }
    pub fn response_arrived_downstream(label: &str) -> Self {
        LogEntry::ResponseArrivedDownstream { label: format!("response arrived at requesting node: {}", &label)  }
    }
    pub fn forward_request_upstream(label: &str) -> Self {
        LogEntry::ForwardRequestUpstream { label: format!("forwarded request from node: {}", &label)  }
    }
    pub fn forward_response_downstream(label: &str) -> Self {
        LogEntry::ForwardResponseDownstream { label: format!("forwarded response from node: {}", &label)  }
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
