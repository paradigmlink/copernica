use {
    client::{CopernicaRequestor},
    crossbeam_channel::{unbounded, Receiver},
    packets::{response},
    faces::{Udp},
    router::{Router, RouterControl},
    std::{
        str::FromStr,
        str,
        thread::{spawn, sleep, JoinHandle},
        time,
        io,
    },
    futures::executor::{ThreadPool},
    fern,
};


#[derive(Debug, PartialEq)]
struct Face {
    listen: String,
    remote: String,
}

#[derive(Debug, PartialEq)]
struct Data {
    name: String,
    data: String,
}

impl FromStr for Face {
    type Err = std::net::AddrParseError;

    fn from_str(socket_pair: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = socket_pair.split('|').collect();
        let listen: String = v[0].to_string();
        let remote: String = v[1].to_string();
        Ok( Face { listen: listen, remote: remote })
    }
}

impl FromStr for Data {
    type Err = std::net::AddrParseError;

    fn from_str(socket_pair: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = socket_pair.split('|').collect();
        let name: String = v[0].to_string();
        let data: String = v[1].to_string();
        Ok( Data { name: name, data: data })
    }

}

fn router(faces: Vec<Face>, data: Option<Vec<Data>>, rx: Receiver<RouterControl>) -> JoinHandle<()> {
    let router = spawn( move || {
        let mut executor = ThreadPool::new().expect("Failed to create threadpool");
        let mut r = Router::new(executor.clone(), rx);
        for face in faces {
            let f = Udp::new(face.listen, face.remote);
            r.add_face(f);
        }
        match data {
            Some(data) => {
                for item in data {
                    r.insert_into_cs(response(item.name, item.data.as_bytes().to_vec()));
                }
            },
            None => {},
        }
        executor.run(r.run())
    });
    router
}

fn simple_fetch() -> packets::Packet {
    let (tx, rx) = unbounded();
    let node0 = vec![Face::from_str("127.0.0.1:8070|127.0.0.1:8071").unwrap(), Face::from_str("127.0.0.1:8072|127.0.0.1:8073").unwrap()];
    router(node0, None, rx.clone());
    let node1 = vec![Face::from_str("127.0.0.1:8073|127.0.0.1:8072").unwrap(), Face::from_str("127.0.0.1:8074|127.0.0.1:8075").unwrap()];
    router(node1, None, rx.clone());
    let node2 = vec![Face::from_str("127.0.0.1:8075|127.0.0.1:8074").unwrap(), Face::from_str("127.0.0.1:8076|127.0.0.1:8077").unwrap()];
    router(node2, None, rx.clone());
    let node3_f = vec![Face::from_str("127.0.0.1:8077|127.0.0.1:8076").unwrap()];
    let node3_d = vec![Data::from_str("hello|world").unwrap()];
    router(node3_f, Some(node3_d), rx.clone());
    sleep(time::Duration::from_millis(1));
    let requestor = CopernicaRequestor::new("127.0.0.1:8071".into(), "127.0.0.1:8070".into());
    let response = requestor.request("hello".into());
    tx.send(RouterControl::Exit).unwrap();
    response
}

fn main() {
    setup_logging(3, None).unwrap();
    simple_fetch();
}

fn setup_logging(verbosity: u64, logpath: Option<&str>) -> Result<(), fern::InitError> {
    let mut base_config = fern::Dispatch::new();

    base_config = match verbosity {
        0 => {
            base_config
                .level(log::LevelFilter::Info)
                .level_for("async_std::task::block_on", log::LevelFilter::Warn)
                .level_for("mio::poll", log::LevelFilter::Warn)
        }
        1 => base_config
            .level(log::LevelFilter::Debug)
            .level_for("mio::poll", log::LevelFilter::Info),
        2 => base_config.level(log::LevelFilter::Debug),
        _3_or_more => base_config
            .level(log::LevelFilter::Trace)
            .level_for("mio::poll", log::LevelFilter::Info)
            .level_for("async_std::task::block_on", log::LevelFilter::Warn),
    };

    // Separate file config so we can include year, month and day in file logs
    let file_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
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
                    "[{}][{}] {}",
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

#[cfg(test)]
mod network_regression_tests {
    use super::*;

    #[test]
    fn a_humble_four_hop_hello_world_fetch() {
        //setup_logging(3, None).unwrap();
        let packet = simple_fetch();
        assert_eq!(response("hello".to_string(), "world".to_string().as_bytes().to_vec()), packet);
    }
}
