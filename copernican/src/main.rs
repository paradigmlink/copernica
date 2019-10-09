
extern crate bincode;
extern crate faces;
//extern crate router;

use faces::{Face, Mock};
use router::router::start;

fn main() {
    let mut router = router::Router::new();
    let f1: Mock = Face::new();
    let f2: Mock = Face::new();
    router.add_face(&f1);
    router.add_face(&f2);
    router.run();
    println!("Copernican");
}
