use {
    anyhow::{anyhow, Result},
    core::hash::{Hash, Hasher},
    std::{
        sync::mpsc::Sender,
        str::FromStr,
        fmt::{self},
    },
    crate::{PublicIdentity, PublicIdentityInterface },
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
    pub fn start(&self) {
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::start()) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn end(&self) {
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::end()) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    // convenience function to reduce typing on Protocol, Link, Router API
    pub fn label(&self, label: &str) -> (String, Self) {
        (label.to_string(), self.clone())
    }
    pub fn register_protocol(&self, id: PublicIdentity, label: String) {
        let attrs = "".into();
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::protocol(id, label, attrs)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn register_link(&self, id: PublicIdentity, label: String) {
        //debug!("{}", LogEntry::link(link_id.link_pid()?, label));
        let attrs = "".into();
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::link(id, label, attrs)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn register_router(&self, id: u32) {
        let attrs = "".into();
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::router(id, attrs)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn link_to_link(&self, from: PublicIdentity, to: PublicIdentityInterface, label: String) {
        let attrs = "".into();
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::pid_to_pid(from, to, label, attrs)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn protocol_to_link(&self, from: PublicIdentity, to: PublicIdentityInterface) {
        let attrs = "".into();
        let label = "".into();
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::pid_to_pid(from, to, label, attrs)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn link_to_protocol(&self, from: PublicIdentity, to: PublicIdentityInterface, label: String) {
        let attrs = "".into();
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::pid_to_pid(from, to, label, attrs)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn link_to_router(&self, from: PublicIdentity, to: u32, face: u32,label: String) {
        let attrs = "".into();
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::pid_to_id(from, to, face, label, attrs)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn router_to_link(&self, from: u32, face: u32, to: PublicIdentity, label: String) {
        let attrs = "".into();
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::id_to_pid(from, face, to, label, attrs)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
    pub fn router_to_router(&self, from: u32, from_face:u32, to: u32, to_face: u32, label: String) {
        let attrs = "".into();
        match self {
            Operations::On { tx } => {
                match tx.send(LogEntry::id_to_id(from, from_face, to, to_face, label, attrs)) {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
            Operations::Off => {}
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogEntry {
    Start,
    End,
    Protocol {
        label: String,
        pid: PublicIdentity,
        attrs: String,
    },
    Router {
        id: u32,
        attrs: String,
    },
    Link {
        label: String,
        pid: PublicIdentity,
        attrs: String,
    },
    Arrow {
        label: String,
        from: ArrowDestination,
        to: ArrowDestination,
        attrs: String,
    },
}
impl LogEntry {
    pub fn start() -> Self {
        LogEntry::Start
    }
    pub fn end() -> Self {
        LogEntry::End
    }
    pub fn protocol(pid: PublicIdentity, label: String, attrs: String) -> Self {
        LogEntry::Protocol { label, pid, attrs}
    }
    pub fn router(id: u32, attrs: String) -> Self {
        LogEntry::Router { id, attrs}
    }
    pub fn link(pid: PublicIdentity, label: String, attrs: String) -> Self {
        LogEntry::Link { pid, label, attrs}
    }
    pub fn pid_to_pid(from: PublicIdentity, to: PublicIdentityInterface, label: String, attrs: String) -> Self {
        LogEntry::Arrow {
            from: ArrowDestination::Identity { id: PublicIdentityInterface::Present { public_identity: from }},
            to: ArrowDestination::Identity { id: to },
            label, attrs
        }
    }
    pub fn id_to_pid(from: u32, face: u32, to: PublicIdentity, label: String, attrs: String) -> Self {
        LogEntry::Arrow {
            from: ArrowDestination::Router { id: from, face },
            to: ArrowDestination::Identity { id: PublicIdentityInterface::Present { public_identity: to}},
            label, attrs
        }
    }
    pub fn pid_to_id(from: PublicIdentity, to: u32, face: u32, label: String, attrs: String) -> Self {
        LogEntry::Arrow {
            from: ArrowDestination::Identity { id: PublicIdentityInterface::Present { public_identity: from }},
            to: ArrowDestination::Router { id: to, face },
            label: label.to_string(), attrs
        }
    }
    pub fn id_to_id(from: u32, from_face: u32, to: u32, to_face: u32, label: String, attrs: String) -> Self {
        LogEntry::Arrow {
            from: ArrowDestination::Router { id: from, face: from_face  },
            to: ArrowDestination::Router { id: to, face: to_face },
            label, attrs
        }
    }
}
impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            LogEntry::Protocol { label, pid, attrs } => {
                format!("|(P,{},{},{})", label, pid, attrs)
            }
            LogEntry::Router { id, attrs } => {
                format!("|(R,{},{})", id, attrs)
            }
            LogEntry::Link { label, pid, attrs } => {
                format!("|(L,{},{},{})", label, pid, attrs)
            }
            LogEntry::Arrow { label, from, to, attrs } => {
                format!("|(A,{},{},{},{})", label, from, to, attrs)
            },
            LogEntry::Start=> {
                format!("|(S)")
            }
            LogEntry::End=> {
                format!("|(E)")
            }
        };
        write!(f, "{}", out)
    }
}
impl FromStr for LogEntry {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if !s.contains("|") { return Err(anyhow!("No \"|\" in the log entry")); }
        let s: Vec<&str> = s.split('|').collect();
        let e: Vec<&str> = s[1].trim_matches(|p| p == '(' || p == ')').split(',').collect();
        let o = match e[0] {
            "P" => LogEntry::Protocol { label: e[1].into(), pid: PublicIdentity::from_str(e[2])?, attrs: e[3].into() },
            "R" => {
                LogEntry::Router { id: e[1].parse::<u32>()?, attrs: e[2].into() }
                },
            "L" => LogEntry::Link { label: e[1].into(), pid: PublicIdentity::from_str(e[2])?, attrs: e[3].into() },
            "A" => LogEntry::Arrow {
                      label: e[1].into(),
                      from: ArrowDestination::from_str(e[2])?,
                      to: ArrowDestination::from_str(e[2])?,
                      attrs: e[3].into()
            },
            "S" => LogEntry::Start,
            "E" => LogEntry::End,
            _ => return Err(anyhow!("Could not parse log entry format")),
        };
        Ok(o)
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArrowDestination {
    Identity { id: PublicIdentityInterface },
    Router{ id: u32, face: u32 },
}
impl fmt::Display for ArrowDestination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match &self {
            ArrowDestination::Identity { id } => {
                format!("(I;{})", id)
            }
            ArrowDestination::Router { id, face} => {
                format!("(R;{};{})", id, face)
            }
        };
        write!(f, "{}", out)
    }
}
impl FromStr for ArrowDestination {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        let e: Vec<&str> = s.trim_matches(|p| p == '(' || p == ')').split(';').collect();
        let e = match e[0] {
            "I" => {
                println!("{:?} {:?}", e, s );
                ArrowDestination::Identity { id: PublicIdentityInterface::Present { public_identity: PublicIdentity::from_str(e[1])? } }
            },
            "R" => ArrowDestination::Router { id: e[1].parse::<u32>()?, face: e[2].parse::<u32>()? },
            _ => return Err(anyhow!("Could not parse ArrowDestination format")),
        };
        Ok(e)
    }
}
impl Hash for ArrowDestination {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ArrowDestination::Identity { id, .. } => { id.hash(state) },
            ArrowDestination::Router { id, face } => { id.hash(state); face.hash(state) },
        }
    }
}
