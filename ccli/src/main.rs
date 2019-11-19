extern crate packets;
use std::env;

use {
    copernica_lib::{
        client::{CopernicaRequestor, load_named_responses},
        crypto::key::generate_identity,
    },
    packets::{response},
    rpassword::prompt_password_stdout,
    bincode::{serialize, deserialize},
    log::{trace},
    structopt::StructOpt,
    std::{
        io,
        io::*,
    },
};

#[derive(StructOpt, Debug)]
#[structopt(name = "ccli", about = "A CLI interface to Copernica, an anonymous content delivery network or networking protocol for the edge of the internet", author = "Stewart Mackenzie <sjm@fractalide.com>", version = "0.1.0")]
struct Options {
    #[structopt(short = "g", long = "generate-id", help = "Generate a new Ed25519 identity keypair")]
    generate_id: bool,

    #[structopt(short = "l", long = "list-ids", help = "List identities")]
    list_ids: bool,

    #[structopt(short = "u", long = "use-id", help = "Load up the private key associated with this identity")]
    use_id: bool,

    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,
}

fn main() {
    let options = Options::from_args();
    let mut home = std::env::home_dir().unwrap();
    home.push(".copernica");
    home.push("identity");
    println!("{:?}", options);

    if options.generate_id {
        let password = prompt_password_stdout("enter your new copernica password: ").unwrap();
        let (addr, material) = generate_identity(password);
        let path = std::path::PathBuf::from(home.clone()).join(addr.clone());
        let material_ser = serialize(&material).unwrap();
        std::fs::write(path, material_ser);
        println!("created identity: {} in {:?}", addr, home);
    }

    if options.list_ids {
        let ids = load_named_responses(home.as_path());
        for (id, res) in ids {
            println!("{}", id);
        }
    }

    if options.use_id {
        let ids = load_named_responses(home.as_path());
        println!("available identities:");
        for (id, res) in ids {
            println!("{}", id);
        }
        let mut chosen_id = String::new();
        println!("select identity:");
        io::stdin().read_line(&mut chosen_id).expect("error: unable to read chosen id");
        let id_password = prompt_password_stdout("enter password for chosen identity: ").unwrap();
        println!("chosen_id: {:?}, id_password: {:?}", chosen_id, id_password);


    }


    //let config = matches.value_of("config").unwrap_or("copernica.json");


    let mut cr = CopernicaRequestor::new("127.0.0.1:8089".into());
    let response = cr.request(vec!["hello".to_string()], 50);
}
