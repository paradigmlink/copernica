use {
    crate::{
        //hbfi::{HBFI},
        borsh::{BorshDeserialize, BorshSerialize},
        link::{Blooms, Nonce, LinkId},
        packets::{InterLinkPacket, WirePacket, NarrowWaist},
        Bayes, LinkWeight
    },
    anyhow::Result,
    //log::{trace},
    crossbeam_channel::Sender,
    log::debug,
    std::collections::HashMap,
};

#[derive(Clone)]
pub struct Router {}

impl Router {
    pub fn handle_packet(
        ilp: &InterLinkPacket,
        r2c_tx: Sender<InterLinkPacket>,
        response_store: sled::Db,
        blooms: &mut HashMap<LinkId, Blooms>,
        bayes: &mut Bayes,
    ) -> Result<()> {
        let this_link: LinkId = ilp.link_id();
        let this_link_id: Nonce = ilp.link_id().nonce();
        let nw: NarrowWaist = ilp.narrow_waist();
        if let Some(this_bloom) = blooms.get_mut(&this_link) {
            match nw.clone() {
                NarrowWaist::Request { hbfi } => {
                    match response_store.get(&hbfi.try_to_vec()?)? {
                        Some(response) => {
                            let nw = NarrowWaist::try_from_slice(&response)?;
                            debug!("********* RESPONSE PACKET FOUND *********");
                            let wp = WirePacket::new(this_link.reply_to(), nw);
                            let ilp = InterLinkPacket::new(this_link.clone(), wp);
                            r2c_tx.send(ilp)?;
                            return Ok(());
                        }
                        None => {
                            debug!("********* NO   RESPONSE   FOUND *********");
                            this_bloom.create_pending_request(&hbfi);
                            for LinkWeight { linkid: that_link, weight: _weight} in bayes.classify(&hbfi.to_vec()) {
                                // meditate on how to utilize weight effectively
                                if that_link.nonce() == this_link_id {
                                    continue;
                                }
                                if let Some(that_bloom) = blooms.get_mut(&that_link) {
                                    if that_bloom.contains_forwarded_request(&hbfi) {
                                        continue;
                                    }
                                    if that_bloom.contains_pending_request(&hbfi) {
                                        continue;
                                    }
                                    that_bloom.create_forwarded_request(&hbfi);
                                    r2c_tx.send(ilp.change_destination(that_link))?;
                                }
                            }
                        }
                    }
                }
                NarrowWaist::Response { hbfi, .. } => {
                    if this_bloom.contains_forwarded_request(&hbfi) {
                        response_store.insert(hbfi.try_to_vec()?, nw.clone().try_to_vec()?)?;
                        bayes.train(&hbfi.to_vec(), &this_link);
                        this_bloom.delete_forwarded_request(&hbfi);
                        for (that_link, that_bloom) in blooms.iter_mut() {
                            if that_link.nonce() == this_link_id {
                                continue;
                            }
                            if that_bloom.contains_pending_request(&hbfi) {
                                that_bloom.delete_pending_request(&hbfi);
                                debug!("********* RESPONSE DOWNSTREAM *********");
                                r2c_tx.send(ilp.change_destination(that_link.clone()))?;
                            }
                        }
                    }
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    }
}
