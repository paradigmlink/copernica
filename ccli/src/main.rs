extern crate packets;
use std::env;

use {
    copernica_lib::{
        client::CopernicaRequestor,
        crypto::key::generate_identity,
    },
    packets::{response},
    rpassword::prompt_password_stdout,
    bincode::{serialize, deserialize},
    log::{trace},
    structopt::StructOpt
};

#[derive(StructOpt, Debug)]
#[structopt(name = "ccli", about = "A CLI interface to Copernica, an anonymous content delivery network or networking protocol for the edge of the internet", author = "Stewart Mackenzie <sjm@fractalide.com>", version = "0.1.0")]
struct Options {
    #[structopt(short = "g", long = "generate-id", help = "Generate a new Ed25519 identity keypair")]
    generate_id: bool,

    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,
}

fn main() {
    let options = Options::from_args();
    println!("{:?}", options);
    if options.generate_id {
        let password = prompt_password_stdout("enter your copernica password: ").unwrap();
        let (addr, material) = generate_identity(password);
        let mut home = std::env::home_dir().unwrap();
        home.push(".copernica");
        home.push("identity");
        let path = std::path::PathBuf::from(home.clone()).join(addr.clone());
        let material_ser = serialize(&material).unwrap();
        std::fs::write(path, material_ser);

        println!("your identity: {} is written to {:?}", addr, home);



    }
//    let config = matches.value_of("config").unwrap_or("copernica.json");
    //let mut cr = CopernicaRequestor::new("127.0.0.1:8089".into());
    //let response = cr.request(vec!["hello".to_string()], 50);
}
