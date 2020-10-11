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
    log::{debug, warn},
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
        deep_six: &LinkId,
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
                            let link_weights = bayes.classify(&hbfi.to_vec());
                            //std::thread::sleep_ms(500);
                            bayes.train(&hbfi.to_vec(), deep_six);
                            if link_weights[0].linkid == *deep_six {
                                warn!("{}, {:?}", link_weights[0].weight, link_weights[0].linkid);
                                let litmus_weight = (link_weights[0].weight * 100.00) as u64;
                                match litmus_weight {
                                    0..=35 => {
                                        warn!("Defcon 4: Do something")
                                    },
                                    36..=59 => {
                                        warn!("Defcon 3: Do something")
                                        // packets need a nonce and it needs to be signed. So as to avert the
                                        // scenario whereby an attacker replays requests thus shutting down
                                        // the flow of legitimate information.
                                    },
                                    60..=89 => {
                                        warn!("Defcon 2: Do something")
                                    },
                                    90..=u64::MAX => {
                                        warn!("Defcon 1: Deep Sixed packet: {:?}", hbfi);
                                        return Ok(())
                                    },
                                }
                            }
                            let mut forwarded = false;
                            for LinkWeight { linkid: that_link, weight} in link_weights {
                                warn!("{}, {:?}", weight, that_link);
                                if that_link == *deep_six {
                                    continue;
                                }
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
                                    if (weight < 0.00) && (forwarded == false) {
                                        that_bloom.create_forwarded_request(&hbfi);
                                        r2c_tx.send(ilp.change_destination(that_link))?;
                                        continue;
                                    }
                                    that_bloom.create_forwarded_request(&hbfi);
                                    r2c_tx.send(ilp.change_destination(that_link))?;
                                    forwarded = true;
                                }
                            }
                        }
                    }
                }
                NarrowWaist::Response { hbfi, .. } => {
                    if this_bloom.contains_forwarded_request(&hbfi) {
                        response_store.insert(hbfi.try_to_vec()?, nw.clone().try_to_vec()?)?;
                        bayes.super_train(&hbfi.to_vec(), &this_link);
                        // ^^^ think about an attack whereby a response is continually sent thus adjusting the weights
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
