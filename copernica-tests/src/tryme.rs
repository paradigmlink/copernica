use std::thread;
use std::time::Duration;
use std::sync::mpsc;
enum Action {
    Timeout,
    Complete,
}
fn loop_it(actions: Vec<Action>) {
    let (send, recv) = mpsc::sync_channel(0);
    thread::spawn(move || {
        let mut counter = 0;
        loop {
            match actions[counter] {
                Action::Timeout => {
                    println!("Timeout");
                    thread::sleep(Duration::from_millis(210));
                },
                Action::Complete => {
                    println!("Complete");
                }
            }
            if counter > 2 {
                send.send(()).unwrap();
                break
            }
            counter += 1;
        }
    });
    let out = recv.recv_timeout(Duration::from_millis(200));
    println!("{:?}", out);
}
fn main() {
    let actions0 = vec![Action::Complete, Action::Timeout, Action::Complete];
    loop_it(actions0);
    let actions1 = vec![Action::Complete, Action::Complete, Action::Complete];
    loop_it(actions1);
    let actions2 = vec![Action::Complete, Action::Complete, Action::Complete];
    loop_it(actions2);
}
