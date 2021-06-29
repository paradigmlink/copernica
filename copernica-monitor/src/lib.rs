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
#[derive(Debug, Clone)]
pub enum GraphVizPlainExt {
    Graph {
        width: u32,
        height: u32,
        fps: u32,
    },
    Node {
        name: String,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        label: String,
        style: String,
        shape: String,
        color: String,
        fillcolor: String,
    },
    //edge tail head n x₁ y₁ .. xₙ yₙ [label xl yl] style color
    Edge{
        name: String,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        label: String,
        style: String,
        shape: String,
        color: String,
        fillcolor: String,
    },

}
// node name x y width height label style shape color fillcolor
// "node id1z6pks9fdd6r3x6nmzzrkjc7pt50r7uhhxg0cevtfr53k284ja6yjpyhdmwjkpk75fhlz4p5murndq2am8q9hw24u8x4x3nvkp3vmcsse6kvn2 0.84028 0.32639 1.6806 0.51389 echo_protocol0 solid record black lightgrey"

impl FromStr for GraphVizPlainExt {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        let e: Vec<&str> = s.split(' ').collect();
        let o = match e[0] {
            "graph" => GraphVizPlainExt::Graph {
                fps: e[1].parse::<f32>()? as u32,
                width: e[2].parse::<f32>()? as u32,
                height: e[3].parse::<f32>()? as u32,
            },
            "node" => GraphVizPlainExt::Node {
                name: e[1].into(),
                x: e[2].parse::<u32>()?,
                y: e[3].parse::<u32>()?,
                width: e[4].parse::<u32>()?,
                height: e[5].parse::<u32>()?,
                label: e[6].into(),
                style: e[7].into(),
                shape: e[8].into(),
                color: e[9].into(),
                fillcolor: e[10].into(),
            },
            /*
            "L" => LogEntry::Link { label: e[1].into(), pid: PublicIdentity::from_str(e[2])?, attrs: e[3].into() },
            "A" => LogEntry::Arrow {
                      label: e[1].into(),
                      from: ArrowDestination::from_str(e[2])?,
                      to: ArrowDestination::from_str(e[2])?,
                      attrs: e[3].into()
            },
            */
            _ => return Err(anyhow!("Could not parse graphviz -Tplain-ext output")),
        };
        Ok(o)
    }
}

