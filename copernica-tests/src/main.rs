mod common;
use {
    anyhow::{Result},
    scaffolding::{ group, scaffold, Ordering},
    copernica_tests::{
        unreliable_sequenced_cleartext_ping_pong, reliable_sequenced_cleartext_ping_pong, reliable_ordered_cleartext_ping_pong,
        unreliable_sequenced_cyphertext_ping_pong, reliable_sequenced_cyphertext_ping_pong, reliable_ordered_cyphertext_ping_pong,
    },
};

fn main() -> Result<()> {
    copernica_common::setup_logging(3, None).unwrap();
      let tests = group!(
        "Network Tests",
        [
            group!(
                "Basic Echo",
                [
                    unreliable_sequenced_cleartext_ping_pong(Ordering::Any),
                    reliable_sequenced_cleartext_ping_pong(Ordering::Any),
                    reliable_ordered_cleartext_ping_pong(Ordering::Any),
                    unreliable_sequenced_cyphertext_ping_pong(Ordering::Any),
                    reliable_sequenced_cyphertext_ping_pong(Ordering::Any),
                    reliable_ordered_cyphertext_ping_pong(Ordering::Any),
                ]
            ),
        ]
    );
    scaffold(tests);
    Ok(())
}
