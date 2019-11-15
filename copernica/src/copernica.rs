extern crate bincode;
extern crate router;
extern crate content_store;
extern crate log;
extern crate serde_derive;
extern crate serde_json;
extern crate clap;

use {
    log::{trace},
    router::{Router, Config},
    logger::setup_logging,
    clap::{Arg, App},
    std::{
        error::Error,
        io::BufReader,
        path::Path,
        fs::File,
    },
};

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
                        .takes_value(true))
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
    r.run();
}

