extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod copernicafs;

pub use self::copernicafs::{
    {Config, CopernicaFacade, FileManager},
    filesystem::{NullFs, CopernicaFs},
};



