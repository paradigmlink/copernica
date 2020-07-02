extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod copernicafs;

pub use copernicafs::filesystem::{NullFs, CopernicaFs};
pub use copernicafs::{Config, CopernicaFacade, FileManager};


