use {
    anyhow::{Result, anyhow},
    bincode,
    copernica_common::{
        bloom_filter_index as bfi, serialization::*,
        NarrowWaistPacket, HBFI, PublicIdentity, PrivateIdentityInterface
    },
    crate::{Protocol, Outbound, Inbound},
    log::debug,
    async_executor::{Executor},
    futures_lite::{future},
};
#[derive(Clone)]
pub struct Echo {
    db: sled::Db,
    protocol_sid: PrivateIdentityInterface,
    outbound: Option<Outbound>,
    inbound: Option<Inbound>,
}
impl<'a> Echo {
    pub fn cleartext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(mut outbound) = self.outbound.clone() {
            let hbfi = HBFI::new(None, response_pid, "echo", "echo", "echo", "echo")?;
            let echo = future::block_on(async { outbound.request2(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
    pub fn cyphertext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        if let Some(mut outbound) = self.outbound.clone() {
            let hbfi = HBFI::new(Some(outbound.protocol_sid.public_id()), response_pid, "echo", "echo", "echo", "echo")?;
            let echo = future::block_on(async { outbound.request2(hbfi.clone(), 0, 0).await });
            let echo: String = bincode::deserialize(&echo?)?;
            Ok(echo)
        } else {
            Err(anyhow!("You must peer with a link first"))
        }
    }
}
impl<'a> Protocol<'a> for Echo {
    fn new(db: sled::Db, protocol_sid: PrivateIdentityInterface) -> Echo {
        Echo {
            db,
            protocol_sid,
            outbound: None,
            inbound: None,
        }
    }
    fn run(&self) -> Result<()> {
        let db = self.db.clone();
        let outbound = self.outbound.clone();
        let inbound = self.inbound.clone();
        let ex = Executor::new();
        let task = ex.spawn(async move {
            if let Some(outbound) = outbound {
                if let Some(inbound) = inbound {
                    let res_check = bfi(&format!("{}", outbound.protocol_sid.clone().public_id()))?;
                    let app_check = bfi("echo")?;
                    let m0d_check = bfi("echo")?;
                    let fun_check = bfi("echo")?;
                    let arg_check = bfi("echo")?;
                    loop {
                        match inbound.clone().next_inbound().await {
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
                                                outbound.clone().respond(hbfi.clone(), echo).await?;
                                            }
                                        _ => {}
                                    },
                                    NarrowWaistPacket::Response { hbfi, .. } => {
                                        debug!("\t\t|  RESPONSE PACKET ARRIVED");
                                        let (_, hbfi_s) = serialize_hbfi(&hbfi)?;
                                        let (_, nw_s) = serialize_narrow_waist_packet(&nw)?;
                                        db.insert(hbfi_s, nw_s)?;
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
    fn set_outbound(&mut self, outbound: Outbound) {
        self.outbound = Some(outbound);
    }
    fn set_inbound(&mut self, inbound: Inbound) {
        self.inbound = Some(inbound);
    }
    fn get_db(&mut self) -> sled::Db {
        self.db.clone()
    }
    fn get_protocol_sid(&mut self) -> PrivateIdentityInterface {
        self.protocol_sid.clone()
    }
}
