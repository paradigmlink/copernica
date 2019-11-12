extern crate bincode;
extern crate router;
extern crate futures;
extern crate content_store;
extern crate log;
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate clap;

use {
    log::{trace},
    router::{Router, Config},
    logger::setup_logging,
    packets::{response},
    futures::executor::{ThreadPool},
    clap::{Arg, App},
    std::{
        str::FromStr,
        io,
        thread::{spawn, park},
        error::Error,
        io::BufReader,
        path::Path,
        fs::File,
    },
    fern,
};

#[derive(Debug, PartialEq)]
struct Face {
    listen: String,
    remote: String,
}

#[derive(Debug, PartialEq)]
struct NamedData {
    name: String,
    data: String,
}

impl FromStr for Face {
    type Err = std::net::AddrParseError;

    fn from_str(socket_pair: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = socket_pair.split('-').collect();
        let listen: String = v[0].to_string();
        let remote: String = v[1].to_string();
        Ok( Face { listen: listen, remote: remote })
    }
}

impl Face {
    fn is_ok(&self) -> bool {
        true
    }
}

fn is_valid_socket(val: String) -> Result<(), String> {
    let face = Face::from_str(&val).unwrap();
    if face.is_ok() {
        Ok(())
    } else {
        Err(String::from("face ip address didn't cut it"))
    }
}

fn read_config_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let confs= serde_json::from_reader(reader)?;
    Ok(confs)
}

fn main() {
    let matches = App::new("Copernica")
                    .version("0.1.0")
                    .author("Stewart Mackenzie <sjm@fractalide.com>")
                    .about("An anonymous content delivery network or networking protocol for the edge of the internet")
                    .arg(Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .help("Path to config file")
                        .takes_value(true))
                    .arg(Arg::with_name("listen")
                        .short("l")
                        .long("listen")
                        .multiple(false)
                        .help("Udp port to listen on")
                        .takes_value(true)
                        .validator(is_valid_socket))
                    .arg(Arg::with_name("verbosity")
                        .short("v")
                        .long("verbosity")
                        .multiple(true)
                        .help("Increases verbosity logging level up to 3 times"),)
                    .get_matches();
    let config = matches.value_of("config").unwrap_or("copernica.json");
    let config = read_config_file(config).unwrap();
    let verbosity: u64 = matches.occurrences_of("verbosity");
    let logpath = matches.value_of("logpath");
    setup_logging(verbosity, logpath).expect("failed to initialize logging.");

    trace!("copernica node started");

    let mut r = Router::new_with_config(config);
    r.add_peer("127.0.0.1:8090".into());
    r.insert_into_cs(response("hello0".into(), "hello0".as_bytes().to_vec()));
    r.insert_into_cs(response("hello1".into(), "hello1".as_bytes().to_vec()));
    r.insert_into_cs(response("hello2".into(), "hello2".as_bytes().to_vec()));
    r.insert_into_cs(response("hello3".into(), "hello3".as_bytes().to_vec()));
    r.insert_into_cs(response("hello4".into(), "hello4".as_bytes().to_vec()));
    r.run();


}

