
extern crate bincode;
extern crate faces;
extern crate router;

fn main() {
    let mut router = router::Router::new();
    router.run();
    router.stop();
    println!("Copernican");
}
