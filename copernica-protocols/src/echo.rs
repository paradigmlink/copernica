use {
    anyhow::{Result, anyhow},
    bincode,
    copernica_common::{
        bloom_filter_index as bfi, NarrowWaistPacket, HBFI, PublicIdentity, PrivateIdentityInterface, InterLinkPacket
    },
    futures::{
        channel::mpsc::{Sender},
        sink::{SinkExt},
    },
    crate::{Protocol, TxRx},
    log::debug,
    async_executor::{Executor},
    futures_lite::{future},
};
#[derive(Clone)]
pub struct Echo {
    protocol_sid: PrivateIdentityInterface,
    txrx: Option<TxRx>,
    i2r_tx: Option<Sender<InterLinkPacket>>,
}
impl<'a> Echo {
    pub fn cleartext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(None, response_pid, "echo", "echo", "echo", "echo")?;
            let echo = future::block_on(async { txrx.request(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn cyphertext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(Some(txrx.protocol_sid.public_id()), response_pid, "echo", "echo", "echo", "echo")?;
            let echo = future::block_on(async { txrx.request(hbfi.clone(), 0, 0).await });
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
            i2r_tx: None,
        }
    }
    fn run(&self) -> Result<()> {
        let txrx = self.txrx.clone();
        let i2r_tx = self.i2r_tx.clone();
        let ex = Executor::new();
        let task = ex.spawn(async move {
            if let Some(txrx) = txrx {
                if let Some(mut i2r_tx) = i2r_tx {
                    let res_check = bfi(&format!("{}", txrx.protocol_sid.clone().public_id()))?;
                    let app_check = bfi("echo")?;
                    let m0d_check = bfi("echo")?;
                    let fun_check = bfi("echo")?;
                    let arg_check = bfi("echo")?;
                    loop {
                        match txrx.clone().next_inbound().await {
                            Some(ilp) => {
                                debug!("\t\t|  link-to-protocol");
                                let nw: NarrowWaistPacket = ilp.narrow_waist();
                                match nw.clone() {
                                    NarrowWaistPacket::Request { hbfi, .. } => match hbfi {
                                        HBFI { res, app, m0d, fun, arg, .. }
                                            if (res == res_check)
                                                && (app == app_check)
                                                && (m0d == m0d_check)
                                                && (fun == fun_check)
                                                && (arg == arg_check)
                                            => {
                                                let echo: Vec<u8> = bincode::serialize(&"pong".to_string())?;
                                                txrx.clone().respond(hbfi.clone(), echo).await?;
                                            }
                                        _ => {}
                                    },
                                    NarrowWaistPacket::Response { .. } => {
                                        debug!("\t\t|  RESPONSE PACKET ARRIVED");
                                        i2r_tx.send(ilp).await?;
                                    }
                                }
                            }
                            None => {}
                        }
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
    fn set_i2r_tx(&mut self, i2r_tx: Sender<InterLinkPacket>) {
        self.i2r_tx = Some(i2r_tx);
    }
    fn get_protocol_sid(&mut self) -> PrivateIdentityInterface {
        self.protocol_sid.clone()
    }
}
