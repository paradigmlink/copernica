use {
    copernica_common::{LinkId, InterLinkPacket, HBFI, PublicIdentity, PrivateIdentityInterface},
    copernica_protocols::{Protocol, Echo},
    std::{thread},
    crossbeam_channel::{Sender, Receiver, unbounded},
    anyhow::{Result, anyhow},
    log::{debug, error},
};
pub struct EchoService {
    link_id: Option<LinkId>,
    db: sled::Db,
    protocol: LOCD,
    sid: PrivateIdentity,
}
impl EchoService {
    pub fn new(db: sled::Db, sid: PrivateIdentity) -> Self {
        let mut protocol: LOCD = Protocol::new();
        Self {
            link_id: None,
            db,
            protocol,
            sid,
        }
    }
    pub fn peer_with_link(
        &mut self,
        link_id: LinkId,
    ) -> Result<(Sender<InterLinkPacket>, Receiver<InterLinkPacket>)> {
        self.link_id = Some(link_id.clone());
        Ok(self.protocol.peer_with_link(self.db.clone(), link_id, self.sid.clone())?)
    }
    pub fn peer_with_node(&mut self, with_identity: PublicIdentity, amount_to_insert_into_multisig: u64, my_limit: u64) -> Result<()> {
        //let contract_details: ContractDetails = self.protocol.contract_details(with_identity.clone())?;
        /*if contract_details.donation_request() > my_limit {
            let msg = "Requested amount is too expensive for what I'm willing to pay";
            error!("{}", msg);
            return Err(anyhow!(msg))
        }*/
        //let their_address = self.protocol.address(with_identity)?;
        //println!("my_limit {}, their_limit {}, their_address {}", my_limit, contract_details.donation_request(), contract_details.address());
        let response = self.protocol.contract_counter_offer(with_identity.clone())?;
        println!("final {:?}", response);
        Ok(())
    }
    pub fn run(&mut self) -> Result<()> {
        self.protocol.run()?;
        Ok(())
    }
}
