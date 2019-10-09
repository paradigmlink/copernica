
extern crate bincode;
extern crate faces;
extern crate router;

use faces::{Face, Mock};

fn main() {
    let mut router = router::Router::new();
    let f1: Mock = Mock::new();
    let f2: Mock = Mock::new();
    router.add_face(f1);
    router.add_face(f2);
    router.run();
    println!("Copernican");
}
