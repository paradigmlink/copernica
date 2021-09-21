use {
    crossbeam_channel::{Sender},
    core::fmt::{self},
    arrayvec::ArrayString,
    crate::constants::LABEL_SIZE,
};
#[derive(Clone, Debug)]
pub enum Operations {
    On {
        tx: Sender<LogEntry>,
        buffer: ArrayString<LABEL_SIZE>,
    },
    Off,
}
impl Operations {
    pub fn turned_on(tx: Sender<LogEntry>) -> Self {
        Operations::On {
            tx,
            buffer: ArrayString::<LABEL_SIZE>::new(),
        }
    }
    pub fn turned_off() -> Self {
        Operations::Off
    }
    // convenience function to reduce typing on Protocol, Link, Router API
    pub fn label(&self, label: &str) -> (ArrayString<LABEL_SIZE>, Self) {
        let mut label_array = ArrayString::<LABEL_SIZE>::new();
        label_array.push_str(label);
        (label_array, self.clone())
    }
    pub fn end(&self) {
        match self {
            Operations::On { tx, .. } => {
                match tx.send(LogEntry::End) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn register_protocol(&self, label: ArrayString<LABEL_SIZE>) {
        match self {
            Operations::On { tx, .. } => {
                match tx.send(LogEntry::register(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn register_link(&self, label: ArrayString<LABEL_SIZE>) {
        match self {
            Operations::On { tx, .. } => {
                match tx.send(LogEntry::register(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn register_router(&self, label: ArrayString<LABEL_SIZE>) {
        match self {
            Operations::On { tx, .. } => {
                match tx.send(LogEntry::register(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn message_from(&self, label: ArrayString<LABEL_SIZE>) {
        match self {
            Operations::On { tx, .. } => {
                match tx.send(LogEntry::message(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn found_response_upstream(&self, label: ArrayString<LABEL_SIZE>) {
        match self {
            Operations::On { tx, .. } => {
                match tx.send(LogEntry::found_response_upstream(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn response_arrived_downstream(&self, label: ArrayString<LABEL_SIZE>) {
        match self {
            Operations::On { tx, .. } => {
                match tx.send(LogEntry::response_arrived_downstream(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn forward_response_downstream(&self, label: ArrayString<LABEL_SIZE>) {
        match self {
            Operations::On { tx, .. } => {
                match tx.send(LogEntry::forward_response_downstream(&label)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn forward_request_upstream(&self, label: ArrayString<LABEL_SIZE>) {
        match self {
            Operations::On { tx, .. } => {
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
    Register(ArrayString<LABEL_SIZE>),
    Message(ArrayString<LABEL_SIZE>),
    FoundResponseUpstream(ArrayString<LABEL_SIZE>),
    ResponseArrivedDownstream(ArrayString<LABEL_SIZE>),
    ForwardResponseDownstream(ArrayString<LABEL_SIZE>),
    ForwardRequestUpstream(ArrayString<LABEL_SIZE>),
}
impl LogEntry {
    pub fn end() -> Self {
        LogEntry::End
    }
    pub fn register(l: &str) -> Self {
        let mut label = ArrayString::<LABEL_SIZE>::new();
        label.push_str("expected_behaviour.insert(LogEntry::register(");
        label.push_str(&l);
        label.push_str(".clone()),");
        LogEntry::Register(label)
    }
    pub fn message(l: &str) -> Self {
        let mut label = ArrayString::<LABEL_SIZE>::new();
        label.push_str("expected_behaviour.insert(LogEntry::message(");
        label.push_str(&l);
        label.push_str(".clone()),");
        LogEntry::Message(label)
    }
    pub fn found_response_upstream(l: &str) -> Self {
        let mut label = ArrayString::<LABEL_SIZE>::new();
        label.push_str("expected_behaviour.insert(LogEntry::found_response_upstream(");
        label.push_str(&l);
        label.push_str(".clone()),");
        LogEntry::FoundResponseUpstream(label)
    }
    pub fn response_arrived_downstream(l: &str) -> Self {
        let mut label = ArrayString::<LABEL_SIZE>::new();
        label.push_str("expected_behaviour.insert(LogEntry::response_arrived_downstream(");
        label.push_str(&l);
        label.push_str(".clone()),");
        LogEntry::ResponseArrivedDownstream(label)
    }
    pub fn forward_request_upstream(l: &str) -> Self {
        let mut label = ArrayString::<LABEL_SIZE>::new();
        label.push_str("expected_behaviour.insert(LogEntry::forward_request_upstream(");
        label.push_str(&l);
        label.push_str(".clone()),");
        LogEntry::ForwardRequestUpstream(label)
    }
    pub fn forward_response_downstream(l: &str) -> Self {
        let mut label = ArrayString::<LABEL_SIZE>::new();
        label.push_str("expected_behaviour.insert(LogEntry::forward_response_downstream(");
        label.push_str(&l);
        label.push_str(".clone()),");
        LogEntry::ForwardResponseDownstream(label)
    }
}
impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut label = ArrayString::<LABEL_SIZE>::new();
        let out = match self {
            LogEntry::Register(label) => { label },
            LogEntry::Message(label) => { label },
            LogEntry::FoundResponseUpstream(label) => { label },
            LogEntry::ResponseArrivedDownstream(label) => { label },
            LogEntry::ForwardResponseDownstream(label) => { label },
            LogEntry::ForwardRequestUpstream(label) => { label },
            LogEntry::End => {
                label.push_str("end");
                &label
            },
        };
        write!(f, "{}", out)
    }
}
