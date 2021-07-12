use {
    crate::{
        bloom_filter::{Blooms},
        Bayes, LinkWeight, ResponseStore
    },
    copernica_common::{LinkId, InterLinkPacket, LinkPacket, NarrowWaistPacket, Operations},
    anyhow::Result,
    std::sync::mpsc::{SyncSender},
    log::{debug, warn},
    std::collections::HashMap,
};
#[derive(Clone)]
pub struct Router {}
impl Router {
    pub fn handle_packet(
        label: &String,
        ops: &Operations,
        ilp: &InterLinkPacket,
        r2b_tx: SyncSender<InterLinkPacket>,
        rs: &mut ResponseStore,
        blooms: &mut HashMap<LinkId, Blooms>,
        bayes: &mut Bayes,
        choke: &LinkId,
    ) -> Result<()> {
        let this_link: LinkId = ilp.link_id();
        let nw: NarrowWaistPacket = ilp.narrow_waist();
        if let Some(this_bloom) = blooms.get_mut(&this_link) {
            match nw.clone() {
                NarrowWaistPacket::Request { hbfi, .. } => {
                    match rs.find(|n|
                        match n {
                            NarrowWaistPacket::Response { hbfi: compare_hbfi, .. } => { compare_hbfi == &hbfi },
                            NarrowWaistPacket::Request  { .. } => { false }
                        }) {

                        Some(nw) => {
                            debug!("\t\t|  |  |  |  RESPONSE PACKET FOUND");
                            ops.found_response(label.clone());
                            let lp = LinkPacket::new(this_link.reply_to()?, nw.clone());
                            let ilp = InterLinkPacket::new(this_link.clone(), lp);
                            r2b_tx.send(ilp)?;
                            return Ok(());
                        }
                        None => {
                            debug!("\t\t|  |  |  |  FORWARD REQUEST UPSTREAM");
                            ops.forward_request_upstream(label.clone());
                            this_bloom.create_pending_request(hbfi.clone());
                            let link_weights = bayes.classify(&hbfi.to_bfis());
                            bayes.train(&hbfi.to_bfis(), choke);
                            if link_weights[0].linkid == *choke {
                                //warn!("{}, {:?}", link_weights[0].weight, link_weights[0].linkid);
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
                                //warn!("{}, {:?}", weight, that_link);
                                if that_link == *choke {
                                    continue;
                                }
                                if that_link.link_pid()? == this_link.link_pid()? {
                                    continue;
                                }
                                if let Some(that_bloom) = blooms.get_mut(&that_link) {
                                    if that_bloom.contains_pending_request(hbfi.clone()) {
                                        continue;
                                    }
                                    if (weight < 0.00) && (forwarded == false) {
                                        that_bloom.create_forwarded_request(hbfi.clone());
                                        r2b_tx.send(ilp.change_destination(that_link))?;
                                        continue;
                                    }
                                    that_bloom.create_forwarded_request(hbfi.clone());
                                    r2b_tx.send(ilp.change_destination(that_link))?;
                                    forwarded = true;
                                }
                            }
                        }
                    }
                }
                NarrowWaistPacket::Response { hbfi, .. } => {
                    if this_bloom.contains_forwarded_request(hbfi.clone()) {
                        rs.insert(nw);
                        bayes.super_train(&hbfi.to_bfis(), &this_link);
                        for (that_link, that_bloom) in blooms.iter_mut() {
                            if that_link.link_pid()? == this_link.link_pid()? {
                                continue;
                            }
                            if that_bloom.contains_pending_request(hbfi.clone()) {
                                debug!("\t\t|  |  |  |  FORWARD RESPONSE DOWNSTREAM");
                                ops.forward_response_downstream(label.clone());
                                r2b_tx.send(ilp.change_destination(that_link.clone()))?;
                            }
                        }
                    }
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    }
}
