use {
    anyhow::{anyhow, Result},
    itertools::join,
    std::{
        str::FromStr,
        fmt::{self},
    },
    copernica_common::{LogEntry, ArrowDestination},
};

#[derive(Clone)]
pub struct DotEntry(pub LogEntry);
impl fmt::Display for DotEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match &self.0 {
            LogEntry::Protocol { label, pid, attrs } => {
                format!("{} [label={}, {}]\n", pid, label, attrs)
            }
            LogEntry::Router { label, ids, attrs } => {
                let ids: Vec<String> = ids.iter().map( |&id| "<".to_string() + &id.to_string() + &"> ".to_string() + &id.to_string()).collect();
                let ids: String = "{".to_owned() + &join (ids, "|") + "}";
                format!("{} [label=\"{}\", {}]\n", label, ids, attrs)
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
pub struct DotArrowDestination(ArrowDestination);
impl fmt::Display for DotArrowDestination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match &self.0 {
            ArrowDestination::Identity { id } => {
                format!("{}", id)
            }
            ArrowDestination::Router { id, face } => {
                format!("{}:{}", id, face)
            }
        };
        write!(f, "{}", out)
    }
}
