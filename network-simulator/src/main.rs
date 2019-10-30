use {
    client::{CopernicaRequestor},
    packets::{Packet, response},
    std::process::Command,
    std::{
        str::from_utf8,
        str,
        env,
        path::{Path, PathBuf},
    },
};

fn router(custom_args: Vec<&str>) -> std::process::Child {
    let mut cargo_manifest_dir_pathbuf = PathBuf::new();
    cargo_manifest_dir_pathbuf.push(Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()));
    cargo_manifest_dir_pathbuf.pop();
    env::set_current_dir(&cargo_manifest_dir_pathbuf);
    let mut copernica = Command::new("rustup");
    let base_args = &["run", "nightly", "cargo", "run", "-q", "--bin", "copernica", "--", "-vvv"];
    let custom_args = custom_args.as_slice();
    let args = [base_args, custom_args].concat();
    copernica.args(args);
    copernica.spawn().expect("failed to spawn")
}

fn simple_fetch() -> packets::Packet {
    // listener-send_to_remote
    let mut procs = vec![];
    let node0 = vec![
        "--face", "127.0.0.1:8070-127.0.0.1:8071", "--face", "127.0.0.1:8072-127.0.0.1:8073",
        "--logpath", "logs/node0.log"];
    procs.push(router(node0));
    let node1 = vec![
        "--face", "127.0.0.1:8073-127.0.0.1:8072", "--face", "127.0.0.1:8074-127.0.0.1:8075",
        "--logpath", "logs/node1.log"];
    procs.push(router(node1));
    let node2 = vec![
        "--face", "127.0.0.1:8075-127.0.0.1:8074", "--face", "127.0.0.1:8076-127.0.0.1:8077",
        "--logpath", "logs/node2.log"];
    procs.push(router(node2));
    let node3 = vec![
        "--face", "127.0.0.1:8077-127.0.0.1:8076",
        "--data", "hello-content",
        "--logpath", "logs/node3.log"];
    procs.push(router(node3));
    use std::{thread, time};

    thread::sleep(time::Duration::from_secs(1));
    let requestor = CopernicaRequestor::new("127.0.0.1:8071".into(), "127.0.0.1:8070".into());
    let response = requestor.request("hello".into());
    for mut proc in procs {
        proc.kill();
    }
    response
}

fn main() {
    simple_fetch();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cs() {
        assert_eq!(response("hello".to_string(), "content".to_string().as_bytes().to_vec()), simple_fetch());
    }
}
