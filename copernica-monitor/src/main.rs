use {
    beard::beard,
    anyhow::{anyhow, Result},
    copernica_common::{PrivateIdentityInterface, PublicIdentity},
    rand::Rng,
    core::hash::{Hash, Hasher},
    std::{
        str::FromStr,
        fmt::{self},
        fs::{OpenOptions},
        io::Write,
    },
};
#[derive(Clone)]
pub struct DotEntry(pub LogEntry);
impl fmt::Display for DotEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match &self.0 {
            LogEntry::Protocol { label, pid, attrs } => {
                format!("{} [label={}, {}]\n", pid, label, attrs)
            }
            LogEntry::Router { label, id, attrs } => {
                format!("{} [label={}, {}]\n", id, label, attrs)
            }
            LogEntry::Link { label, pid, attrs } => {
                format!("{} [label={}, {}]\n", pid, label, attrs)
            }
            LogEntry::Arrow {from, to, .. } => {
                format!("{} -> {}\n", DotArrowDestination(from.clone()), DotArrowDestination(to.clone()))
            },
        };
        write!(f, "{}", out)
    }
}
#[derive(Clone)]
pub enum ArrowDestination {
    Identity { id: PublicIdentity },
    Router{ id: u32 },
}
impl fmt::Display for ArrowDestination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match &self {
            ArrowDestination::Identity { id } => {
                format!("(I,{})", id)
            }
            ArrowDestination::Router { id } => {
                format!("(R,{})", id)
            }
        };
        write!(f, "{}", out)
    }
}
#[derive(Clone)]
pub struct DotArrowDestination(ArrowDestination);
impl fmt::Display for DotArrowDestination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match &self.0 {
            ArrowDestination::Identity { id } => {
                format!("{}", id)
            }
            ArrowDestination::Router { id } => {
                format!("{}", id)
            }
        };
        write!(f, "{}", out)
    }
}
impl FromStr for ArrowDestination {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        let e: Vec<&str> = s.trim_matches(|p| p == '(' || p == ')').split(',').collect();
        let e = match e[0] {
            "I" => ArrowDestination::Identity { id: PublicIdentity::from_str(e[2])? },
            "R" => ArrowDestination::Router { id: e[2].parse::<u32>()? },
            _ => return Err(anyhow!("Could not parse ArrowDestination format")),
        };
        Ok(e)
    }
}
impl Hash for ArrowDestination {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ArrowDestination::Identity { id, .. } => { id.hash(state) },
            ArrowDestination::Router { id, .. } => { id.hash(state) },
        }
    }
}
#[derive(Clone)]
pub enum LogEntry {
    Protocol {
        label: String,
        pid: PublicIdentity,
        attrs: String,
    },
    Router {
        label: String,
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
    pub fn protocol(pid: PublicIdentity, label: String) -> Self {
        let attrs = "".into();
        LogEntry::Protocol { label, pid, attrs}
    }
    pub fn router(id: u32, label: String) -> Self {
        let attrs = "".into();
        LogEntry::Router { id, label, attrs}
    }
    pub fn link(pid: PublicIdentity, label: String) -> Self {
        let attrs = "".into();
        LogEntry::Link { pid, label, attrs}
    }
    pub fn link_to_link(from: PublicIdentity, to: PublicIdentity, label: String) -> Self {
        let attrs = "".into();
        LogEntry::Arrow {
            from: ArrowDestination::Identity { id: from },
            to: ArrowDestination::Identity { id: to },
            label, attrs
        }
    }
    pub fn protocol_to_link(from: PublicIdentity, to: PublicIdentity, label: String) -> Self {
        let attrs = "".into();
        LogEntry::Arrow {
            from: ArrowDestination::Identity { id: from },
            to: ArrowDestination::Identity { id: to },
            label, attrs
        }
    }
    pub fn link_to_protocol(from: PublicIdentity, to: PublicIdentity, label: String) -> Self {
        let attrs = "".into();
        LogEntry::Arrow {
            from: ArrowDestination::Identity { id: from },
            to: ArrowDestination::Identity { id: to },
            label, attrs
        }
    }
    pub fn link_to_router(from: PublicIdentity, to: u32, label: String) -> Self {
        let attrs = "".into();
        LogEntry::Arrow {
            from: ArrowDestination::Identity { id: from },
            to: ArrowDestination::Router{ id: to },
            label, attrs
        }
    }
    pub fn router_to_link(from: u32, to: PublicIdentity, label: String) -> Self {
        let attrs = "".into();
        LogEntry::Arrow {
            from: ArrowDestination::Router{ id: from },
            to: ArrowDestination::Identity { id: to },
            label, attrs
        }
    }
    pub fn router_to_router(from: u32, to: u32, label: String) -> Self {
        let attrs = "".into();
        LogEntry::Arrow {
            from: ArrowDestination::Router{ id: to },
            to: ArrowDestination::Router { id: from },
            label, attrs
        }
    }
}
impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            LogEntry::Protocol { label, pid, attrs } => {
                format!("(P,{},{},{})", label, pid, attrs)
            }
            LogEntry::Router { label, id, attrs } => {
                format!("(R,{},{},{})", label, id, attrs)
            }
            LogEntry::Link { label, pid, attrs } => {
                format!("(L,{},{},{})", label, pid, attrs)
            }
            LogEntry::Arrow { label, from, to, attrs } => {
                format!("(A,{},{},{},{})", label, from, to, attrs)
            },
        };
        write!(f, "{}", out)
    }
}
impl FromStr for LogEntry {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        let e: Vec<&str> = s.trim_matches(|p| p == '(' || p == ')').split(',').collect();
        let e = match e[0] {
            "P" => LogEntry::Protocol { label: e[1].into(), pid: PublicIdentity::from_str(e[2])?, attrs: e[3].into() },
            "R" => LogEntry::Router { label: e[1].into(), id: e[2].parse::<u32>()?, attrs: e[3].into() },
            "L" => LogEntry::Link { label: e[1].into(), pid: PublicIdentity::from_str(e[2])?, attrs: e[3].into() },
            "A" => LogEntry::Arrow {
                      label: e[1].into(),
                      from: ArrowDestination::from_str(e[2])?,
                      to: ArrowDestination::from_str(e[2])?,
                      attrs: e[3].into()
            },
            _ => return Err(anyhow!("Could not parse log entry format")),
        };
        Ok(e)
    }
}
fn main() -> Result<()> {
    let mut queue: Vec<LogEntry> = vec![];
    let mut rng = rand::thread_rng();
    let protocol_sid0 = PrivateIdentityInterface::new_key();
    let protocol0 = LogEntry::protocol(protocol_sid0.public_id(), "protocol0".into());
    let link_sid0 = PrivateIdentityInterface::new_key();
    let link0 = LogEntry::link(link_sid0.public_id(), "link0".into());
    let link_sid1 = PrivateIdentityInterface::new_key();
    let link1 = LogEntry::link(link_sid1.public_id(), "link1".into());
    let link_sid2 = PrivateIdentityInterface::new_key();
    let link2 = LogEntry::link(link_sid2.public_id(), "link2".into());
    let link_sid3 = PrivateIdentityInterface::new_key();
    let link3 = LogEntry::link(link_sid3.public_id(), "link3".into());
    let arrow0 = LogEntry::protocol_to_link(protocol_sid0.public_id(), link_sid0.public_id(), "".into());
    let arrow1 = LogEntry::link_to_link(link_sid0.public_id(), link_sid1.public_id(), "".into());
    let router_id0 = rng.gen::<u32>();
    let router= LogEntry::router(router_id0, "router0".into());
    let arrow2 = LogEntry::link_to_router(link_sid1.public_id(), router_id0, "".into());
    let arrow3 = LogEntry::router_to_link(router_id0, link_sid2.public_id(), "".into());
    let arrow4 = LogEntry::link_to_link(link_sid2.public_id(), link_sid3.public_id(), "".into());
    let protocol_sid1 = PrivateIdentityInterface::new_key();
    let protocol1 = LogEntry::protocol(protocol_sid1.public_id(), "protocol1".into());
    let arrow5 = LogEntry::link_to_protocol(link_sid3.public_id(), protocol_sid1.public_id(), "".into());
    queue.push(protocol0);
    queue.push(link0);
    queue.push(link1);
    queue.push(link2);
    queue.push(link3);
    queue.push(arrow0);
    queue.push(arrow1);
    queue.push(router);
    queue.push(arrow2);
    queue.push(arrow3);
    queue.push(arrow4);
    queue.push(protocol1);
    queue.push(arrow5);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("monitor.dot")?;
    beard! {
        file,
        "
digraph G {
  rankdir=TB;
  node [shape=record];\n"
  for entry in ( queue ) {
      { DotEntry(entry) }
  }
"
}
"
    };
    Ok(())
}
