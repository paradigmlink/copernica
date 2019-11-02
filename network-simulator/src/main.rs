use {
    client::{CopernicaClient},
    crossbeam_channel::{unbounded, Receiver},
    packets::{response, request},
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
    log::{trace},
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

fn router(faces: Vec<Face>, data: Option<Vec<Data>>, ctl_recv: Receiver<RouterControl>) -> JoinHandle<()> {
    let router = spawn( move || {
        let mut executor = ThreadPool::new().expect("Failed to create threadpool");
        let mut r = Router::new(executor.clone(), ctl_recv);
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
    let mut executor: ThreadPool = ThreadPool::new().unwrap();
    let (ctl_send, ctl_recv) = unbounded();
    let node0 = vec![Face::from_str("127.0.0.1:8070|127.0.0.1:8071").unwrap(), Face::from_str("127.0.0.1:8072|127.0.0.1:8073").unwrap()];
    router(node0, None, ctl_recv.clone());
    let node1 = vec![Face::from_str("127.0.0.1:8073|127.0.0.1:8072").unwrap(), Face::from_str("127.0.0.1:8074|127.0.0.1:8075").unwrap()];
    router(node1, None, ctl_recv.clone());
    let node2 = vec![Face::from_str("127.0.0.1:8075|127.0.0.1:8074").unwrap(), Face::from_str("127.0.0.1:8076|127.0.0.1:8077").unwrap()];
    router(node2, None, ctl_recv.clone());
    let node3_f = vec![Face::from_str("127.0.0.1:8077|127.0.0.1:8076").unwrap()];
    let node3_d = vec![Data::from_str("hello|world").unwrap()];
    router(node3_f, Some(node3_d), ctl_recv.clone());
    sleep(time::Duration::from_millis(10));
    let (cc, inbound) = CopernicaClient::new("127.0.0.1:8071".into(), "127.0.0.1:8070".into());
    let ccc = cc.clone();
    std::thread::spawn( move || { executor.run(ccc.inbound()) });
    cc.outbound(request("hello".into()));
    sleep(time::Duration::from_millis(10));
    let response = inbound.recv().unwrap();
    ctl_send.send(RouterControl::Exit).unwrap();
    response
}

fn small_small_world_graph() -> packets::Packet {
    // https://en.wikipedia.org/wiki/File:Small-world-network-example.png
    let mut executor: ThreadPool = ThreadPool::new().unwrap();
    let (ctl_send, ctl_recv) = unbounded(); // node 0 is the top node in the diagram, node 1 is clockwise one in the diagram
    let node0 = vec![Face::from_str("127.0.0.1:50000|127.0.0.1:50001").unwrap(),    Face::from_str("127.0.0.1:50002|127.0.0.1:50003").unwrap(),
                     Face::from_str("127.0.0.1:50029|127.0.0.1:50030").unwrap(),    // 0 -> 2
                     Face::from_str("127.0.0.1:50033|127.0.0.1:50034").unwrap(),    // 0 -> 3
                     Face::from_str("127.0.0.1:50037|127.0.0.1:50038").unwrap(),    // 0 -> 5
                     Face::from_str("127.0.0.1:50039|127.0.0.1:50040").unwrap(),    // 0 -> 7
                     Face::from_str("127.0.0.1:50046|127.0.0.1:50045").unwrap(),    // 0 -> 9
                     Face::from_str("127.0.0.1:50050|127.0.0.1:50049").unwrap()];   // 0 -> 10
    router(node0, None, ctl_recv.clone());
    let node1 = vec![Face::from_str("127.0.0.1:50003|127.0.0.1:50002").unwrap(),    Face::from_str("127.0.0.1:50004|127.0.0.1:50005").unwrap()];
    let data1 = vec![Data::from_str("hello1|world").unwrap()];
    router(node1, Some(data1), ctl_recv.clone());
    let node2 = vec![Face::from_str("127.0.0.1:50005|127.0.0.1:50004").unwrap(),    Face::from_str("127.0.0.1:50006|127.0.0.1:50007").unwrap(),
                     Face::from_str("127.0.0.1:50030|127.0.0.1:50029").unwrap(),    // 2 -> 0
                     Face::from_str("127.0.0.1:50031|127.0.0.1:50032").unwrap()];   // 2 -> 4
    router(node2, None, ctl_recv.clone());
    let node3 = vec![Face::from_str("127.0.0.1:50007|127.0.0.1:50006").unwrap(),    Face::from_str("127.0.0.1:50008|127.0.0.1:50009").unwrap(),
                     Face::from_str("127.0.0.1:50034|127.0.0.1:50033").unwrap(),    // 3 -> 0
                     Face::from_str("127.0.0.1:50035|127.0.0.1:50036").unwrap()];   // 3 -> 7
    router(node3, None, ctl_recv.clone());
    let node4 = vec![Face::from_str("127.0.0.1:50009|127.0.0.1:50008").unwrap(),    Face::from_str("127.0.0.1:50010|127.0.0.1:50011").unwrap(),
                     Face::from_str("127.0.0.1:50032|127.0.0.1:50031").unwrap()];   // 4 -> 2
    router(node4, None, ctl_recv.clone());
    let node5 = vec![Face::from_str("127.0.0.1:50011|127.0.0.1:50010").unwrap(),    Face::from_str("127.0.0.1:50012|127.0.0.1:50013").unwrap(),
                     Face::from_str("127.0.0.1:50038|127.0.0.1:50037").unwrap()];   // 5 -> 0
    router(node5, None, ctl_recv.clone());
    let node6 = vec![Face::from_str("127.0.0.1:50013|127.0.0.1:50012").unwrap(),    Face::from_str("127.0.0.1:50014|127.0.0.1:50015").unwrap(),
                     Face::from_str("127.0.0.1:50027|127.0.0.1:50028").unwrap(),    // 6 -> requestor1
                     Face::from_str("127.0.0.1:50041|127.0.0.1:50042").unwrap()];   // 6 -> 8
    router(node6, None, ctl_recv.clone());
    let node7 = vec![Face::from_str("127.0.0.1:50015|127.0.0.1:50014").unwrap(),    Face::from_str("127.0.0.1:50016|127.0.0.1:50017").unwrap(),
                     Face::from_str("127.0.0.1:50036|127.0.0.1:50035").unwrap(),    // 7 -> 3
                     Face::from_str("127.0.0.1:50040|127.0.0.1:50039").unwrap(),    // 7 -> 0
                     Face::from_str("127.0.0.1:50043|127.0.0.1:50044").unwrap(),    // 7 -> 9
                     Face::from_str("127.0.0.1:50047|127.0.0.1:50048").unwrap()];   // 7 -> 10
    router(node7, None, ctl_recv.clone());
    let node8 = vec![Face::from_str("127.0.0.1:50017|127.0.0.1:50016").unwrap(),    Face::from_str("127.0.0.1:50018|127.0.0.1:50019").unwrap(),
                     Face::from_str("127.0.0.1:50042|127.0.0.1:50041").unwrap()];   // 8 -> 6
    router(node8, None, ctl_recv.clone());
    let node9 = vec![Face::from_str("127.0.0.1:50019|127.0.0.1:50018").unwrap(),    Face::from_str("127.0.0.1:50020|127.0.0.1:50021").unwrap(),
                     Face::from_str("127.0.0.1:50044|127.0.0.1:50043").unwrap(),    // 9 -> 7
                     Face::from_str("127.0.0.1:50045|127.0.0.1:50046").unwrap()];   // 9 -> 0
    router(node9, None, ctl_recv.clone());
    let node10 = vec![Face::from_str("127.0.0.1:50021|127.0.0.1:50020").unwrap(),   Face::from_str("127.0.0.1:50022|127.0.0.1:50023").unwrap(),
                      Face::from_str("127.0.0.1:50048|127.0.0.1:50047").unwrap(),   // 10 -> 7
                      Face::from_str("127.0.0.1:50051|127.0.0.1:50052").unwrap(),   // 10 -> requestor2
                      Face::from_str("127.0.0.1:50049|127.0.0.1:50050").unwrap()];  // 10 -> 0
    router(node10, None, ctl_recv.clone());
    let node11 = vec![Face::from_str("127.0.0.1:50023|127.0.0.1:50022").unwrap(), Face::from_str("127.0.0.1:50001|127.0.0.1:50000").unwrap()];
    //let data11 = vec![Data::from_str("hello11|world").unwrap()];
    router(node11, None, ctl_recv.clone());
    sleep(time::Duration::from_millis(10));
    let (cc, inbound) = CopernicaClient::new("127.0.0.1:50028".into(), "127.0.0.1:50027".into());
    let ccc = cc.clone();
    std::thread::spawn( move || { executor.run(ccc.inbound()) });
    //cc.outbound(request("hello1".into()));
    cc.outbound(request("hello1".into()));
    let response = inbound.recv().unwrap();
    ctl_send.send(RouterControl::Exit).unwrap();
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
mod network_regressions {
    use super::*;

    #[test]
    fn a_simple_fetch() {
        setup_logging(3, None).unwrap();
        let packet = simple_fetch();
        assert_eq!(response("hello".to_string(), "world".to_string().as_bytes().to_vec()), packet);
    }

    #[test]
    fn a_small_small_world_graph() {
        setup_logging(3, None).unwrap();
        let packet = small_small_world_graph();
        assert_eq!(response("hello1".to_string(), "world".to_string().as_bytes().to_vec()), packet);
    }
}
