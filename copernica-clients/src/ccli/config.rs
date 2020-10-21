use {
    structopt::StructOpt,
};

#[derive(StructOpt, Debug)]
#[structopt(name = "ccli", about = "A CLI interface to Copernica, an anonymous content delivery network or networking protocol for the edge of the internet", author = "Stewart Mackenzie <sjm@fractalide.com>", version = "0.1.0")]
pub struct Options {
    #[structopt(short = "g", long = "generate-id", help = "Generate a new Ed25519 identity keypair")]
    pub generate_id: bool,

    #[structopt(short = "d", long = "decrypt-id", help = "Generate a new Ed25519 identity keypair")]
    pub decrypt_id: Option<String>,

    #[structopt(short = "l", long = "list-ids", help = "List identities")]
    pub list_ids: bool,

    #[structopt(short = "u", long = "use-id", help = "Load up the private key associated with this identity")]
    pub use_id: bool,

    #[structopt(short = "t", long = "trust-id", help = "Trust someone else's identity -t <your-id> <their-id> <another-id> <etc>")]
    pub trust_id: Option<Vec<String>>,

    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: u8,

    #[structopt(short = "c", long = "config", help = "Location of your config file")]
    pub config: Option<String>,

    #[structopt(short = "p", long = "publish", help = "Publish material")]
    pub publish: Option<String>,

    #[structopt(short = "D", long = "destination", help = "Destination")]
    pub destination: Option<String>,
}
