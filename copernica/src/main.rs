
extern crate bincode;
extern crate faces;
extern crate router;

use faces::{Udp};
use router::{Router};
fn main() {
    let mut r = Router::new();
    let f1 = Udp::new("127.0.0.1:8090".to_string(), "127.0.0.1:8091".to_string());
    let f2 = Udp::new("127.0.0.1:8091".to_string(), "127.0.0.1:8092".to_string());
    let f3 = Udp::new("127.0.0.1:8092".to_string(), "127.0.0.1:8093".to_string());
    r.add_face(f1);
    r.add_face(f2);
    r.add_face(f3);
    r.run();
}
