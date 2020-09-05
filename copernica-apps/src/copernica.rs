use {
    log::{trace},
    copernica::{Copernica, LinkId, ReplyTo},
    transport::{MpscChannel, UdpIp, Transport},
    logger::setup_logging,
    clap::{Arg, App},
    //async_std::{ task, },
    anyhow::{Result},
    std::{
        fs,
        env,
        path::PathBuf,
    },
};

pub fn generate_random_dir_name() -> PathBuf {
    use std::iter;
    use rand::{Rng, thread_rng};
    use rand::distributions::Alphanumeric;

    let mut rng = thread_rng();
    let unique_dir: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(7)
            .collect();

    let mut dir = env::temp_dir();
    dir.push("copernica");
    dir.push(unique_dir);
    fs::create_dir_all(dir.clone()).unwrap();
    dir
}

fn main() -> Result<()> {
    let matches = App::new("Copernica")
                    .version("0.1.0")
                    .author("Stewart Mackenzie <sjm@fractalide.com>")
                    .about("An anonymous content delivery network or networking protocol for the edge of the internet")
                    .arg(Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .help("Path to config file")
                        .takes_value(true))
                    .arg(Arg::with_name("verbosity")
                        .short("v")
                        .long("verbosity")
                        .multiple(true)
                        .help("Increases verbosity logging level up to 3 times"),)
                    .get_matches();
    let _config = matches.value_of("config").unwrap_or("copernica.json");
    let verbosity: u64 = matches.occurrences_of("verbosity");
    let logpath = matches.value_of("logpath");
    setup_logging(verbosity, logpath).expect("failed to initialize logging.");

    trace!("copernica node started");

    let mut c0 = Copernica::new();
    let mut c1 = Copernica::new();

    let c0l0 = LinkId::new(ReplyTo::Mpsc, 0);
    let c1l0 = LinkId::new(ReplyTo::Mpsc, 0);
    let c1l1 = LinkId::new(ReplyTo::Mpsc, 0);
    let c1l2 = LinkId::new(ReplyTo::UdpIp("127.0.0.1:50099".parse()?), 0);

    let mut mpsc0: MpscChannel = Transport::new(c0l0.clone(), c0.create_link(c0l0)?);
    let mut mpsc1: MpscChannel = Transport::new(c1l0.clone(), c1.create_link(c1l0)?);
    let mut mpsc2: MpscChannel = Transport::new(c1l1.clone(), c1.create_link(c1l1)?);
    let udpip: UdpIp       = Transport::new(c1l2.clone(), c1.create_link(c1l2)?);

    mpsc0.peer_with(mpsc1.peer_info());
    mpsc1.peer_with(mpsc0.peer_info());

    mpsc0.peer_with(mpsc2.peer_info());
    mpsc2.peer_with(mpsc0.peer_info());

    mpsc0.run()?;
    mpsc1.run()?;
    mpsc2.run()?;
    udpip.run()?;

    let rd0 = generate_random_dir_name();
    let rd1 = generate_random_dir_name();
    println!("{:?}, {:?}", rd0, rd1);
    let rs0 = sled::open(rd0)?;
    let rs1 = sled::open(rd1)?;

    std::thread::spawn( move || {
        c0.run(rs0)?;
        Ok::<(), anyhow::Error>(())
    });
    c1.run(rs1)?;
    Ok(())
}
