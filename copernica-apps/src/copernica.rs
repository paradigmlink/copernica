use {
    log::{trace},
    copernica_logger::setup_logging,
    copernica_libs::{
        CopernicaApp, FileSharer
    },
    copernica_core::{
        Copernica, LinkId, ReplyTo
    },
    copernica_links::{Link, MpscChannel },
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

    let drop_hook = Box::new(move || {});
    let dir0 = generate_random_dir_name();
    let dir1 = generate_random_dir_name();
    let rs0 = sled::open(dir0)?;
    let rs1 = sled::open(dir1)?;
    let mut c0 = Copernica::new();
    let mut c1 = Copernica::new();
    let lid0 = LinkId::listen(ReplyTo::Mpsc);
    let lid1 = LinkId::listen(ReplyTo::Mpsc);
    let mut mpscchannel0: MpscChannel = Link::new(lid0.clone(), c0.peer(lid0)?)?;
    let mut mpscchannel1: MpscChannel = Link::new(lid1.clone(), c1.peer(lid1)?)?;
    mpscchannel0.female(mpscchannel1.male());
    mpscchannel1.female(mpscchannel0.male());
    let ts0: Vec<Box<dyn Link>> = vec![Box::new(mpscchannel0)];
    let ts1: Vec<Box<dyn Link>> = vec![Box::new(mpscchannel1)];
    let mut fs0: FileSharer = CopernicaApp::new(rs0, drop_hook.clone());
    let mut fs1: FileSharer = CopernicaApp::new(rs1, drop_hook);
    fs0.start(c0, ts0)?;
    fs1.start(c1, ts1)?;
    Ok(())
}

