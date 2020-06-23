use {
    anyhow::{ Result },
    log::{trace},
    libcopernica::{
        Router, CopernicaRequestor,
        Config,
        constants,
        narrow_waist::{Bytes},
        response_store::{Response, mk_response},
    },
    logger::setup_logging,
    clap::{Arg, App},
    nix::{
        unistd,
        sys::stat,
    },
    async_std::{ task, },
    dirs,
    tempdir::TempDir,
    std::{
        fs::File,
        io::{BufRead, BufReader},
    },
    pipe::{Pipe},
    crossbeam_channel::{
        unbounded,
        Sender,
        Receiver,
        select,
        after,
        never
    },
};
mod pipe;

fn main() -> Result<()> {
    let matches = App::new("Takeaway")
                    .version("0.1.0")
                    .author("Stewart Mackenzie <sjm@fractalide.com>")
                    .about("A file sharing application built on Copernica, a content delivery network or networking protocol for the edge of the internet")
                    .arg(Arg::with_name("mnt")
                        .short("m")
                        .long("mount")
                        .help("Path to mount file system")
                        .takes_value(true))
                    .arg(Arg::with_name("verbosity")
                        .short("v")
                        .long("verbosity")
                        .multiple(true)
                        .help("Increases verbosity logging level up to 3 times"),)
                    .get_matches();
    let verbosity: u64 = matches.occurrences_of("verbosity");
    let logpath = matches.value_of("logpath");
    setup_logging(verbosity, logpath).expect("failed to initialize logging.");

    trace!("takeaway started");

    let (cli_s, cli_r) = unbounded();
    std::thread::spawn(move || {
        if let Some(home_dir) = dirs::home_dir() {
            let pipe = Pipe::new(home_dir.join("takeaway.pipe"));
            match pipe {
                Ok(mut p) => {
                    let f = p.file();
                    let mut b = String::new();
                    let mut reader = BufReader::new(f);
                    loop {
                        let data = reader.read_line(&mut b).unwrap();
                        if data != 0 {
                            cli_s.send(b.clone());
                            b.clear();
                        }
                    }
                }
                Err(e) => println!("{}", e),
            }
        }
    });
    task::block_on(async {
        loop{
            match cli_r.recv() {
                Ok(query) => {
                    println!("{}", query);
                    let mut cc = CopernicaRequestor::new("127.0.0.1:50099".into(), "127.0.0.1:50100".into());
                    let retries: u8 = 0;
                    let timeout_per_retry: u64 = 10;
                    cc.start_polling();

                    let size0: usize = 1024;
                    let expected_hello0 = mk_response(query.clone(), vec![0; size0]);
                    let actual_hello0 = cc.request(query, retries, timeout_per_retry).await;
                    assert_eq!(actual_hello0, Some(expected_hello0));
                },
                _ => {},
            }
        }
    });
    Ok(())
}
