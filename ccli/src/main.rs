extern crate serde_derive;
extern crate serde_json;

use {
    copernica::{
        client::{Requestor, FileSharer, FilePacker},
        narrow_waist::{NarrowWaist},
        transport::{ReplyTo},
        //identity::{generate_identity,
          //decrypt_identity
        //},
        //web_of_trust::{add_trusted_identity},
        Config, read_config_file,
    },
    sled,
    anyhow::{Result},
    borsh::{BorshDeserialize},
    rpassword::prompt_password_stdout,
    structopt::StructOpt,
    std::{
        collections::{HashMap},
        path::{Path, PathBuf},
        io,
    },
};
mod config;

fn main() -> Result<()> {
    let options = config::Options::from_args();
    let mut config = Config::new();
    if let Some(config_path) = options.config {
        config = read_config_file(config_path).unwrap();
    }
    let listen_addr = ReplyTo::Udp("127.0.0.1:8089".parse().unwrap());
    let remote_addr = ReplyTo::Udp("127.0.0.1:8089".parse().unwrap());
    let dir: PathBuf = config.data_dir.clone().into();
    let rs: sled::Db = sled::open(dir)?;
    let mut cr: FileSharer = Requestor::new(rs, listen_addr, remote_addr);
    cr.start_polling();
    // stick in the config to the above
/*
    if options.generate_id {
        let password = prompt_password_stdout("enter your new copernica password: ").unwrap();
        generate_identity(password, &config)?;
    }
*/
    if options.list_ids {
        let mut identity_dir = std::path::PathBuf::from(&config.data_dir);
        identity_dir.push(".copernica");
        identity_dir.push("identity");
        let ids = load_named_responses(&identity_dir)?;
        for (id, _res) in ids {
            println!("{}", id);
        }
    }

    if options.use_id {
        let mut identity_dir = std::path::PathBuf::from(&config.data_dir);
        identity_dir.push(".copernica");
        identity_dir.push("identity");
        let ids = load_named_responses(&identity_dir)?;
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
/*
    if let Some(id) = options.decrypt_id {
        let password = prompt_password_stdout("enter password for chosen identity: ").unwrap();

        if let Some(id) = cr.request(id.to_string(), 100).await {
            let digest = String::from_utf8(id.payload()).unwrap();
            println!("{:?}", decrypt_identity(password, digest).unwrap());
        }
    }

    if let Some(ids) = options.trust_id {
        let password = prompt_password_stdout("enter password for chosen identity: ").unwrap();

        if let Some((id_name, rest)) = ids.split_first() {
            let id = cr.request(id_name.to_string(), 100);

            if let Some(Some(id_packet)) = id.get(id_name) {

                //let _id = add_trusted_identity(password, id_packet.clone(), rest.to_vec());
            }
        }
    }
*/

    //let config = matches.value_of("config").unwrap_or("copernica.json");

    if let Some(publish_path) = options.publish {
        if let Some(destination) = options.destination {
            let publish_path= std::path::PathBuf::from(&publish_path);
            let destination_path = std::path::PathBuf::from(&destination);

            let p = FilePacker::new(&publish_path, &destination_path)?;
            p.publish()?;
        }
    }

    Ok(())
}

fn load_named_responses(dir: &Path) -> Result<HashMap<String, NarrowWaist>> {
    let mut resps: HashMap<String, NarrowWaist> = HashMap::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            continue
        } else {
            let contents = std::fs::read(path.clone())?;
            let packet: NarrowWaist = NarrowWaist::try_from_slice(&contents)?;
            let name = &path.file_stem().unwrap();
            resps.insert(name.to_os_string().into_string().unwrap(), packet);
        }
    }
    Ok(resps)
}
