extern crate serde_derive;
extern crate serde_json;

use {
    copernica_lib::{
        client::{CopernicaRequestor, load_named_responses},
        identity::{generate_identity, decrypt_identity},
        //web_of_trust::{add_trusted_identity},
        Config, read_config_file,
    },
    rpassword::prompt_password_stdout,
    structopt::StructOpt,
    std::{
        io,
    },
};

#[derive(StructOpt, Debug)]
#[structopt(name = "ccli", about = "A CLI interface to Copernica, an anonymous content delivery network or networking protocol for the edge of the internet", author = "Stewart Mackenzie <sjm@fractalide.com>", version = "0.1.0")]
struct Options {
    #[structopt(short = "g", long = "generate-id", help = "Generate a new Ed25519 identity keypair")]
    generate_id: bool,

    #[structopt(short = "d", long = "decrypt-id", help = "Generate a new Ed25519 identity keypair")]
    decrypt_id: Option<String>,

    #[structopt(short = "l", long = "list-ids", help = "List identities")]
    list_ids: bool,

    #[structopt(short = "u", long = "use-id", help = "Load up the private key associated with this identity")]
    use_id: bool,

    #[structopt(short = "t", long = "trust-id", help = "Trust someone else's identity -t <your-id> <their-id> <another-id> <etc>")]
    trust_id: Option<Vec<String>>,

    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,

    #[structopt(short = "c", long = "config", help = "Location of your config file")]
    config: Option<String>,
}

fn main() {
    let options = Options::from_args();
    let mut config = Config::new();
    if let Some(config_path) = options.config {
        config = read_config_file(config_path).unwrap();
    }
    let mut cr = CopernicaRequestor::new("127.0.0.1:8089".into());
    // stick in the config to the above

    if options.generate_id {
        let password = prompt_password_stdout("enter your new copernica password: ").unwrap();
        generate_identity(password, &config);
    }

    if options.list_ids {
        let mut identity_dir = std::path::PathBuf::from(&config.data_dir);
        identity_dir.push(".copernica");
        identity_dir.push("identity");
        let ids = load_named_responses(&identity_dir);
        for (id, _res) in ids {
            println!("{}", id);
        }
    }

    if options.use_id {
        let mut identity_dir = std::path::PathBuf::from(&config.data_dir);
        identity_dir.push(".copernica");
        identity_dir.push("identity");
        let ids = load_named_responses(&identity_dir);
        println!("available identities:");
        for (id, _res) in ids {
            println!("{}", id);
        }
        let mut chosen_id = String::new();
        println!("select identity:");
        io::stdin().read_line(&mut chosen_id).expect("error: unable to read chosen id");
        let id_password = prompt_password_stdout("enter password for chosen identity: ").unwrap();
        println!("chosen_id: {:?}, id_password: {:?}", chosen_id, id_password);
    }

    if let Some(id) = options.decrypt_id {
        let password = prompt_password_stdout("enter password for chosen identity: ").unwrap();

        let id = cr.resolve(id.to_string(), 100);
        let digest = String::from_utf8(id.to_vec()).unwrap();
        println!("{:?}", decrypt_identity(password, digest).unwrap());
    }

    if let Some(ids) = options.trust_id {
        let password = prompt_password_stdout("enter password for chosen identity: ").unwrap();

        if let Some((id_name, rest)) = ids.split_first() {
            let id = cr.request(vec![id_name.to_string()], 100);

            if let Some(Some(id_packet)) = id.get(id_name) {

                //let _id = add_trusted_identity(password, id_packet.clone(), rest.to_vec());
            }
        }
    }


    //let config = matches.value_of("config").unwrap_or("copernica.json");


}
