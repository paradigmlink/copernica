use {
    anyhow::{Result, anyhow},
    bincode,
    copernica_common::{
        bloom_filter_index as bfi, NarrowWaistPacket, HBFI, PublicIdentity, PrivateIdentityInterface
    },
    futures::{
        sink::{SinkExt},
    },
    crate::{Protocol, TxRx},
    log::debug,
    async_executor::{Executor},
    futures_lite::{future},
};
static UNRELIABLE_UNORDERED_ECHO: &str = "unreliable_unordered_echo";
static UNRELIABLE_SEQUENCED_ECHO: &str = "unreliable_sequenced_echo";
static RELIABLE_UNORDERED_ECHO: &str = "reliable_unordered_echo";
static RELIABLE_ORDERED_ECHO: &str = "reliable_ordered_echo";
static RELIABLE_SEQUENCED_ECHO: &str = "reliable_sequenced_echo";
#[derive(Clone)]
pub struct Echo {
    protocol_sid: PrivateIdentityInterface,
    txrx: Option<TxRx>,
}
impl<'a> Echo {
    pub fn unreliable_unordered_cleartext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(None, response_pid, "echo", "echo", "echo", UNRELIABLE_UNORDERED_ECHO)?;
            let echo = future::block_on(async { txrx.unreliable_unordered_request(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn unreliable_unordered_cyphertext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(Some(txrx.protocol_sid.public_id()), response_pid, "echo", "echo", "echo", UNRELIABLE_UNORDERED_ECHO)?;
            let echo = future::block_on(async { txrx.unreliable_unordered_request(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn unreliable_sequenced_cleartext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(None, response_pid, "echo", "echo", "echo", UNRELIABLE_SEQUENCED_ECHO)?;
            let echo = future::block_on(async { txrx.unreliable_sequenced_request(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn unreliable_sequenced_cyphertext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(Some(txrx.protocol_sid.public_id()), response_pid, "echo", "echo", "echo", UNRELIABLE_SEQUENCED_ECHO)?;
            let echo = future::block_on(async { txrx.unreliable_sequenced_request(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn reliable_unordered_cleartext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(None, response_pid, "echo", "echo", "echo", RELIABLE_UNORDERED_ECHO)?;
            let echo = future::block_on(async { txrx.reliable_unordered_request(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn reliable_unordered_cyphertext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(Some(txrx.protocol_sid.public_id()), response_pid, "echo", "echo", "echo", RELIABLE_UNORDERED_ECHO)?;
            let echo = future::block_on(async { txrx.reliable_unordered_request(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn reliable_ordered_cleartext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(None, response_pid, "echo", "echo", "echo", RELIABLE_ORDERED_ECHO)?;
            let echo = future::block_on(async { txrx.reliable_ordered_request(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn reliable_ordered_cyphertext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(Some(txrx.protocol_sid.public_id()), response_pid, "echo", "echo", "echo", RELIABLE_ORDERED_ECHO)?;
            let echo = future::block_on(async { txrx.reliable_ordered_request(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn reliable_sequenced_cleartext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(None, response_pid, "echo", "echo", "echo", RELIABLE_SEQUENCED_ECHO)?;
            let echo = future::block_on(async { txrx.reliable_sequenced_request(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn reliable_sequenced_cyphertext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(Some(txrx.protocol_sid.public_id()), response_pid, "echo", "echo", "echo", RELIABLE_SEQUENCED_ECHO)?;
            let echo = future::block_on(async { txrx.reliable_sequenced_request(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
}
impl<'a> Protocol<'a> for Echo {
    fn new(protocol_sid: PrivateIdentityInterface) -> Echo {
        Echo {
            protocol_sid,
            txrx: None,
        }
    }
    fn run(&self) -> Result<()> {
        let txrx = self.txrx.clone();
        let ex = Executor::new();
        let task = ex.spawn(async move {
            if let Some(txrx) = txrx {
                let mut unreliable_unordered_response = txrx.unreliable_unordered_response_tx.clone();
                let mut unreliable_sequenced_response = txrx.unreliable_sequenced_response_tx.clone();
                let mut reliable_unordered_response = txrx.reliable_unordered_response_tx.clone();
                let mut reliable_ordered_response = txrx.reliable_ordered_response_tx.clone();
                let mut reliable_sequenced_response = txrx.reliable_sequenced_response_tx.clone();
                let res_check = bfi(&format!("{}", txrx.protocol_sid.clone().public_id()))?;
                let app_check = bfi("echo")?;
                let m0d_check = bfi("echo")?;
                let fun_check = bfi("echo")?;
                loop {
                    match txrx.clone().next_inbound().await {
                        Some(ilp) => {
                            debug!("\t\t|  link-to-protocol");
                            let nw: NarrowWaistPacket = ilp.narrow_waist();
                            match nw.clone() {
                                NarrowWaistPacket::Request { hbfi, .. } => match hbfi {
                                    HBFI { res, app, m0d, fun, arg, ost, .. }
                                        if (res == res_check)
                                            && (app == app_check)
                                            && (m0d == m0d_check)
                                            && (fun == fun_check)
                                        => {
                                            match arg {
                                                arg if arg == bfi(UNRELIABLE_UNORDERED_ECHO)? => {
                                                    let echo: Vec<u8> = bincode::serialize(&"pong".to_string())?;
                                                    txrx.clone().respond(hbfi.clone(), echo).await?;
                                                },
                                                arg if arg == bfi(UNRELIABLE_SEQUENCED_ECHO)? => {
                                                    let mut echo: Vec<u8> = bincode::serialize(&"pang".to_string())?;
                                                    match ost {
                                                        ost if ost == 0 => {
                                                            echo = bincode::serialize(&"p".to_string())?;
                                                        }
                                                        ost if ost == 1 => {
                                                            echo = bincode::serialize(&"o".to_string())?;
                                                        }
                                                        ost if ost == 2 => {
                                                            echo = bincode::serialize(&"n".to_string())?;
                                                        }
                                                        ost if ost == 3 => {
                                                            echo = bincode::serialize(&"g".to_string())?;
                                                        }
                                                        _ => {}
                                                    }
                                                    txrx.clone().respond(hbfi.clone(), echo).await?;
                                                },
                                                arg if arg == bfi(RELIABLE_UNORDERED_ECHO)? => {
                                                    let echo: Vec<u8> = bincode::serialize(&"pong".to_string())?;
                                                    txrx.clone().respond(hbfi.clone(), echo).await?;
                                                },
                                                arg if arg == bfi(RELIABLE_ORDERED_ECHO)? => {
                                                    let echo: Vec<u8> = bincode::serialize(&"pong".to_string())?;
                                                    txrx.clone().respond(hbfi.clone(), echo).await?;
                                                },
                                                arg if arg == bfi(RELIABLE_SEQUENCED_ECHO)? => {
                                                    let echo: Vec<u8> = bincode::serialize(&"pong".to_string())?;
                                                    txrx.clone().respond(hbfi.clone(), echo).await?;
                                                },
                                                _ => {}
                                            }
                                        }
                                    _ => {}
                                },
                                NarrowWaistPacket::Response { hbfi, .. } => match hbfi {
                                    HBFI { app, m0d, fun, arg, .. }
                                        if (app == app_check)
                                            && (m0d == m0d_check)
                                            && (fun == fun_check)
                                        => {
                                            match arg {
                                                arg if arg == bfi(UNRELIABLE_UNORDERED_ECHO)? => {
                                                    debug!("\t\t|  RESPONSE PACKET ARRIVED");
                                                    unreliable_unordered_response.send(ilp).await?;
                                                },
                                                arg if arg == bfi(UNRELIABLE_SEQUENCED_ECHO)? => {
                                                    debug!("\t\t|  RESPONSE PACKET ARRIVED");
                                                    unreliable_sequenced_response.send(ilp).await?;
                                                },
                                                arg if arg == bfi(RELIABLE_UNORDERED_ECHO)? => {
                                                    debug!("\t\t|  RESPONSE PACKET ARRIVED");
                                                    reliable_unordered_response.send(ilp).await?;
                                                },
                                                arg if arg == bfi(RELIABLE_ORDERED_ECHO)? => {
                                                    debug!("\t\t|  RESPONSE PACKET ARRIVED");
                                                    reliable_ordered_response.send(ilp).await?;
                                                },
                                                arg if arg == bfi(RELIABLE_SEQUENCED_ECHO)? => {
                                                    debug!("\t\t|  RESPONSE PACKET ARRIVED");
                                                    reliable_sequenced_response.send(ilp).await?;
                                                },
                                                _ => {}
                                            }
                                        }
                                    _ => {}
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        std::thread::spawn(move || future::block_on(ex.run(task)));
        Ok(())
    }
    fn set_txrx(&mut self, txrx: TxRx) {
        self.txrx = Some(txrx);
    }
    fn get_protocol_sid(&mut self) -> PrivateIdentityInterface {
        self.protocol_sid.clone()
    }
}
