use {
    anyhow::{Result, anyhow},
    bincode,
    copernica_common::{
        bloom_filter_index as bfi, serialization::*,
        NarrowWaistPacket, HBFI, PublicIdentity
    },
    crate::{Protocol, TxRx},
    log::debug,
    std::thread,
};
#[derive(Clone)]
pub struct Echo {
    txrx: Option<TxRx>,
}
impl<'a> Echo {
    pub fn cleartext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(None, response_pid, "echo", "echo", "echo", "echo")?;
            let echo = txrx.request2(hbfi.clone(), 0, 0)?;
            let echo: String = bincode::deserialize(&echo)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn cyphertext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(txrx) = self.txrx.clone() {
            let hbfi = HBFI::new(Some(txrx.protocol_sid.public_id()), response_pid, "echo", "echo", "echo", "echo")?;
            let echo = txrx.request2(hbfi.clone(), 0, 0)?;
            let echo: String = bincode::deserialize(&echo)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
}
impl<'a> Protocol<'a> for Echo {
    fn new() -> Echo {
        Echo {
            txrx: None,
        }
    }
    fn run(&mut self) -> Result<()> {
        let txrx = self.get_txrx();
        thread::spawn(move || {
            if let Some(txrx) = txrx {
                let res_check = bfi(&format!("{}", txrx.protocol_sid.clone().public_id()))?;
                let app_check = bfi("echo")?;
                let m0d_check = bfi("echo")?;
                let fun_check = bfi("echo")?;
                let arg_check = bfi("echo")?;
                loop {
                    if let Ok(ilp) = txrx.l2p_rx.recv() {
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
                                        txrx.respond(hbfi.clone(), echo)?;
                                    }
                                _ => {}
                            },
                            NarrowWaistPacket::Response { hbfi, .. } => {
                                debug!("\t\t|  RESPONSE PACKET ARRIVED");
                                let (_, hbfi_s) = serialize_hbfi(&hbfi)?;
                                let (_, nw_s) = serialize_narrow_waist_packet(&nw)?;
                                txrx.db.insert(hbfi_s, nw_s)?;
                            }
                        }
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }
    fn set_txrx(&mut self, txrx: TxRx) {
        self.txrx = Some(txrx);
    }
    fn get_txrx(&mut self) -> Option<TxRx> {
        self.txrx.clone()
    }
}
