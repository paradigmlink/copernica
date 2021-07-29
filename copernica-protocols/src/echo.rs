use {
    anyhow::{Result},
    bincode,
    copernica_common::{
        bloom_filter_index as bfi, NarrowWaistPacket, HBFI, PublicIdentity, PublicIdentityInterface, PrivateIdentityInterface, Operations
    },
    crate::{Protocol, TxRx},
    log::{trace},
};
static UNRELIABLE_SEQUENCED_ECHO: &str = "unreliable_sequenced_echo";
static RELIABLE_ORDERED_ECHO: &str = "reliable_ordered_echo";
static RELIABLE_SEQUENCED_ECHO: &str = "reliable_sequenced_echo";
#[derive(Clone)]
pub struct Echo {
    label: String,
    protocol_sid: PrivateIdentityInterface,
    txrx: TxRx,
    ops: Operations,
}
impl Echo {
    pub fn unreliable_sequenced_cleartext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        let hbfi = HBFI::new(PublicIdentityInterface::Absent, response_pid, "echo", "echo", "echo", UNRELIABLE_SEQUENCED_ECHO)?;
        let mut retries = 5;
        let mut window_timeout = 500;
        let echo: Vec<Vec<u8>> = self.txrx.unreliable_sequenced_request(hbfi.clone(), 0, 7, &mut retries, &mut window_timeout)?;
        let mut result: String = "".into();
        for s in &echo {
            let data: &str = bincode::deserialize(&s)?;
            result.push_str(data);
        }
        Ok(result)
    }
    pub fn unreliable_sequenced_cyphertext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        let hbfi = HBFI::new(PublicIdentityInterface::new(self.txrx.protocol_pid()?), response_pid, "echo", "echo", "echo", UNRELIABLE_SEQUENCED_ECHO)?;
        let mut retries = 5;
        let mut window_timeout = 500;
        let echo: Vec<Vec<u8>> = self.txrx.unreliable_sequenced_request(hbfi.clone(), 0, 7, &mut retries, &mut window_timeout)?;
        let mut result: String = "".into();
        for s in &echo {
            let data: &str = bincode::deserialize(&s)?;
            result.push_str(data);
        }
        Ok(result)
    }
    pub fn reliable_ordered_cleartext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        let hbfi = HBFI::new(PublicIdentityInterface::Absent, response_pid, "echo", "echo", "echo", RELIABLE_ORDERED_ECHO)?;
        let mut retries = 5;
        let mut window_timeout = 500;
        let echo: Vec<Vec<u8>> = self.txrx.reliable_ordered_request(hbfi.clone(), 0, 7, &mut retries, &mut window_timeout)?;
        let mut result: String = "".into();
        for s in &echo {
            let data: &str = bincode::deserialize(&s)?;
            result.push_str(data);
        }
        Ok(result)
    }
    pub fn reliable_ordered_cyphertext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        let hbfi = HBFI::new(PublicIdentityInterface::new(self.txrx.protocol_pid()?), response_pid, "echo", "echo", "echo", RELIABLE_ORDERED_ECHO)?;
        let mut retries = 5;
        let mut window_timeout = 500;
        let echo: Vec<Vec<u8>> = self.txrx.reliable_ordered_request(hbfi.clone(), 0, 7, &mut retries, &mut window_timeout)?;
        let mut result: String = "".into();
        for s in &echo {
            let data: &str = bincode::deserialize(&s)?;
            result.push_str(data);
        }
        Ok(result)
    }
    pub fn reliable_sequenced_cleartext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        let hbfi = HBFI::new(PublicIdentityInterface::Absent, response_pid, "echo", "echo", "echo", RELIABLE_SEQUENCED_ECHO)?;
        let mut retries = 5;
        let mut window_timeout = 500;
        let echo: Vec<Vec<u8>> = self.txrx.reliable_sequenced_request(hbfi.clone(), 0, 7, &mut retries, &mut window_timeout)?;
        let mut result: String = "".into();
        for s in &echo {
            let data: &str = bincode::deserialize(&s)?;
            result.push_str(data);
        }
        Ok(result)
    }
    pub fn reliable_sequenced_cyphertext_ping(&mut self, response_pid: PublicIdentity) -> Result<String> {
        let hbfi = HBFI::new(PublicIdentityInterface::new(self.txrx.protocol_pid()?), response_pid, "echo", "echo", "echo", RELIABLE_SEQUENCED_ECHO)?;
        let mut retries = 5;
        let mut window_timeout = 500;
        let echo: Vec<Vec<u8>> = self.txrx.reliable_sequenced_request(hbfi.clone(), 0, 7, &mut retries, &mut window_timeout)?;
        let mut result: String = "".into();
        for s in &echo {
            let data: &str = bincode::deserialize(&s)?;
            result.push_str(data);
        }
        Ok(result)
    }
}
impl Protocol for Echo {
    fn new(protocol_sid: PrivateIdentityInterface, (label, ops): (String, Operations)) -> Self {
        ops.register_protocol(label.clone());
        Self {
            label,
            protocol_sid,
            txrx: TxRx::Inert,
            ops,
        }
    }
    #[allow(unreachable_code)]
    #[allow(unused_assignments)]
    fn run(&self) -> Result<()> {
        let txrx = self.txrx.clone();
        let ops = self.ops.clone();
        let label = self.label.clone();
        std::thread::spawn(move || {
            let res_check = bfi(&format!("{}", txrx.protocol_pid()?))?;
            let app_check = bfi("echo")?;
            let m0d_check = bfi("echo")?;
            let fun_check = bfi("echo")?;
            loop {
                match txrx.next() {
                    Ok(ilp) => {
                        ops.message_from(label.clone());
                        trace!("\t\t|  link-to-protocol");
                        let nw: NarrowWaistPacket = ilp.narrow_waist();
                        match nw.clone() {
                            NarrowWaistPacket::Request { hbfi, .. } => match hbfi {
                                HBFI { res, app, m0d, fun, arg, frm, .. }
                                    if (res == res_check)
                                        && (app == app_check)
                                        && (m0d == m0d_check)
                                        && (fun == fun_check)
                                    => {
                                        match arg {
                                            arg if arg == bfi(UNRELIABLE_SEQUENCED_ECHO)? => {
                                                let mut echo: Vec<u8> = vec![];
                                                match frm {
                                                    frm if frm == 0 => {
                                                        echo = bincode::serialize(&"p")?;
                                                    }
                                                    frm if frm == 1 => {
                                                        echo = bincode::serialize(&"i")?;
                                                    }
                                                    frm if frm == 2 => {
                                                        echo = bincode::serialize(&"n")?;
                                                    }
                                                    frm if frm == 3 => {
                                                        echo = bincode::serialize(&"g")?;
                                                    }
                                                    frm if frm == 4 => {
                                                        echo = bincode::serialize(&"p")?;
                                                    }
                                                    frm if frm == 5 => {
                                                        echo = bincode::serialize(&"o")?;
                                                    }
                                                    frm if frm == 6 => {
                                                        echo = bincode::serialize(&"n")?;
                                                    }
                                                    frm if frm == 7 => {
                                                        echo = bincode::serialize(&"g")?;
                                                    }
                                                    _ => {
                                                        echo = bincode::serialize(&"pang")?;
                                                    }
                                                }
                                                txrx.respond(hbfi, echo)?;
                                            },
                                            arg if arg == bfi(RELIABLE_ORDERED_ECHO)? => {
                                                let mut echo: Vec<u8> = vec![];
                                                match frm {
                                                    frm if frm == 0 => {
                                                        echo = bincode::serialize(&"p")?;
                                                    }
                                                    frm if frm == 1 => {
                                                        echo = bincode::serialize(&"i")?;
                                                    }
                                                    frm if frm == 2 => {
                                                        echo = bincode::serialize(&"n")?;
                                                    }
                                                    frm if frm == 3 => {
                                                        echo = bincode::serialize(&"g")?;
                                                    }
                                                    frm if frm == 4 => {
                                                        echo = bincode::serialize(&"p")?;
                                                    }
                                                    frm if frm == 5 => {
                                                        echo = bincode::serialize(&"o")?;
                                                    }
                                                    frm if frm == 6 => {
                                                        echo = bincode::serialize(&"n")?;
                                                    }
                                                    frm if frm == 7 => {
                                                        echo = bincode::serialize(&"g")?;
                                                    }
                                                    _ => {
                                                        echo = bincode::serialize(&"pang")?;
                                                    }
                                                }
                                                txrx.respond(hbfi, echo)?;
                                            },
                                            arg if arg == bfi(RELIABLE_SEQUENCED_ECHO)? => {
                                                let mut echo: Vec<u8> = vec![];
                                                match frm {
                                                    frm if frm == 0 => {
                                                        echo = bincode::serialize(&"p")?;
                                                    }
                                                    frm if frm == 1 => {
                                                        echo = bincode::serialize(&"i")?;
                                                    }
                                                    frm if frm == 2 => {
                                                        echo = bincode::serialize(&"n")?;
                                                    }
                                                    frm if frm == 3 => {
                                                        echo = bincode::serialize(&"g")?;
                                                    }
                                                    frm if frm == 4 => {
                                                        echo = bincode::serialize(&"p")?;
                                                    }
                                                    frm if frm == 5 => {
                                                        echo = bincode::serialize(&"o")?;
                                                    }
                                                    frm if frm == 6 => {
                                                        echo = bincode::serialize(&"n")?;
                                                    }
                                                    frm if frm == 7 => {
                                                        echo = bincode::serialize(&"g")?;
                                                    }
                                                    _ => {
                                                        echo = bincode::serialize(&"pang")?;
                                                    }
                                                }
                                                txrx.respond(hbfi, echo)?;
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
                                            arg if arg == bfi(UNRELIABLE_SEQUENCED_ECHO)? => {
                                                trace!("\t\t|  RESPONSE PACKET ARRIVED");
                                                ops.response_arrived_downstream(label.clone());
                                                txrx.unreliable_sequenced_response(ilp)?;
                                            },
                                            arg if arg == bfi(RELIABLE_ORDERED_ECHO)? => {
                                                trace!("\t\t|  RESPONSE PACKET ARRIVED");
                                                ops.response_arrived_downstream(label.clone());
                                                txrx.reliable_ordered_response(ilp)?;
                                            },
                                            arg if arg == bfi(RELIABLE_SEQUENCED_ECHO)? => {
                                                trace!("\t\t|  RESPONSE PACKET ARRIVED");
                                                ops.response_arrived_downstream(label.clone());
                                                txrx.reliable_sequenced_response(ilp)?;
                                            },
                                            _ => {}
                                        }
                                    }
                                _ => {}
                            }
                        }
                    }
                    Err(_e) => {}
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }
    fn set_txrx(&mut self, txrx: TxRx) {
        self.txrx = txrx;
    }
    fn get_protocol_sid(&mut self) -> PrivateIdentityInterface {
        self.protocol_sid.clone()
    }
    fn get_ops(&self) -> Operations {
        self.ops.clone()
    }
    fn get_label(&self) -> String {
        self.label.clone()
    }
}
