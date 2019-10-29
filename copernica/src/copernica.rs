extern crate bincode;
extern crate faces;
extern crate router;
extern crate futures;
extern crate content_store;
extern crate log;
#[macro_use]
extern crate clap;
use {
    log::{trace},
    faces::{Udp},
    router::{Router},
    packets::{response},
    futures::executor::{ThreadPool},
    clap::{Arg, App},
    std::{
        str::FromStr,
        io,
        thread::{spawn, park},
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

impl FromStr for NamedData {
    type Err = std::net::AddrParseError;

    fn from_str(socket_pair: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = socket_pair.split('-').collect();
        let name: String = v[0].to_string();
        let data: String = v[1].to_string();
        Ok( NamedData { name: name, data: data })
    }

}
impl Face {
    fn is_ok(&self) -> bool {
        true
    }
}

impl NamedData {
    fn is_ok(&self) -> bool {
        true
    }
}

fn is_face_socket_pair_valid(val: String) -> Result<(), String> {
    let face = Face::from_str(&val).unwrap();
    if face.is_ok() {
        Ok(())
    } else {
        Err(String::from("face ip address didn't cut it"))
    }
}

fn is_data_valid(val: String) -> Result<(), String> {
    let named_data = NamedData::from_str(&val).unwrap();
    if named_data.is_ok() {
        Ok(())
    } else {
        Err(String::from("face ip address didn't cut it"))
    }
}
fn main() {
    let matches = App::new("Copernica")
                    .version("0.1.0")
                    .author("Stewart Mackenzie <sjm@fractalide.com>")
                    .about("An anonymous content delivery network or networking protocol for the edge of the internet")
                    .arg(Arg::with_name("logpath")
                        .short("l")
                        .long("logpath")
                        .help("Path of the logfile")
                        .takes_value(true))
                    .arg(Arg::with_name("face")
                        .short("f")
                        .long("face")
                        .multiple(true)
                        .help("A face consisting of a listening ipaddress and port and remote ipaddress and port e.g. 127.0.0.1:8080-127.0.0.1:8081")
                        .takes_value(true)
                        .required(true)
                        .validator(is_face_socket_pair_valid))
                    .arg(Arg::with_name("data")
                        .short("d")
                        .long("data")
                        .multiple(true)
                        .help("Initialize a router with named data. e.g. key-value or name_of_data-the_actual_data. Separate with a hyphen. For testing purposes")
                        .takes_value(true)
                        .required(true)
                        .validator(is_data_valid))
                    .arg(Arg::with_name("verbosity")
                        .short("v")
                        .long("verbosity")
                        .multiple(true)
                        .help("Increases verbosity logging level up to 3 times"),)
                    .get_matches();
    let _config = matches.value_of("config").unwrap_or("default.conf");
    let verbosity: u64 = matches.occurrences_of("verbosity");
    let logpath = matches.value_of("logpath");
    setup_logging(verbosity, logpath).expect("failed to initialize logging.");

    trace!("copernica started");

    spawn( move || {
        let mut executor = ThreadPool::new().expect("Failed to create threadpool");
        let mut r = Router::new(executor.clone());
        for face in values_t!(matches, "face", Face).unwrap_or_else(|e| e.exit()) {
            trace!("adding face");
            let f = Udp::new(face.listen, face.remote);
            r.add_face(f);
        }
        for named_data in values_t!(matches, "data", NamedData).unwrap_or_else(|e| e.exit()) {
            trace!("adding named data");
            r.insert_into_cs(response(named_data.name, named_data.data.as_bytes().to_vec()));
        }
        executor.run(r.run())
    });

    park();

}

fn setup_logging(verbosity: u64, logpath: Option<&str>) -> Result<(), fern::InitError> {
    let mut base_config = fern::Dispatch::new();

    base_config = match verbosity {
        0 => {
            base_config
                .level(log::LevelFilter::Info)
                .level_for("mio::poll", log::LevelFilter::Warn)
        }
        1 => base_config
            .level(log::LevelFilter::Debug)
            .level_for("mio::poll", log::LevelFilter::Info),
        2 => base_config.level(log::LevelFilter::Debug),
        _3_or_more => base_config
            .level(log::LevelFilter::Trace)
            .level_for("mio::poll", log::LevelFilter::Info),
    };

    // Separate file config so we can include year, month and day in file logs
    let file_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file(logpath.unwrap_or("copernica.log"))?);

    let stdout_config = fern::Dispatch::new()
        .format(|out, message, record| {
            // special format for debug messages coming from our own crate.
            if record.level() > log::LevelFilter::Info && record.target() == "cmd_program" {
                out.finish(format_args!(
                    "---\nDEBUG: {}: {}\n---",
                    chrono::Local::now().format("%H:%M:%S"),
                    message
                ))
            } else {
                out.finish(format_args!(
                    "[{}][{}][{}] {}",
                    chrono::Local::now().format("%H:%M"),
                    record.target(),
                    record.level(),
                    message
                ))
            }
        })
        .chain(io::stdout());

    base_config
        .chain(file_config)
        .chain(stdout_config)
        .apply()?;

    Ok(())
}
