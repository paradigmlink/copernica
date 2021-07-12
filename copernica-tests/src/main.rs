mod common;
use {
    anyhow::{Result},
    scaffolding::{ group, scaffold, Ordering},
    copernica_tests::{network_echo},
};

fn main() -> Result<()> {
    copernica_common::setup_logging(3, None).unwrap();
      let tests = group!(
        "Network Tests",
        [
            group!(
                "Basic Echo",
                [
                    network_echo(Ordering::Any),
                ]
            ),
        ]
    );
    scaffold(tests);
    Ok(())
}
