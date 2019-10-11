
extern crate bincode;
extern crate faces;
extern crate router;

use faces::{Tcp};

fn main() {
    let mut router = router::Router::new();
    let f1 = Tcp::new();
    let f2 = Tcp::new();
    router.add_face(f1);
    router.add_face(f2);
    router.run();
    println!("Copernican");
}
