use {
    crate::{
        //hbfi::{HBFI},
        borsh::{BorshDeserialize, BorshSerialize},
        link::{Blooms, Link, LinkId},
        packets::{InterLinkPacket, WirePacket, NarrowWaist},
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
        blooms: &mut HashMap<Link, Blooms>,
    ) -> Result<()> {
        let this_link: Link = ilp.link();
        let this_link_id: LinkId = ilp.link().id();
        let nw: NarrowWaist = ilp.narrow_waist();
        if let Some(this_bloom) = blooms.get_mut(&this_link) {
            match nw.clone() {
                NarrowWaist::Request { hbfi } => {
                    match response_store.get(&hbfi.try_to_vec()?)? {
                        Some(response) => {
                            let nw = NarrowWaist::try_from_slice(&response)?;
                            //outbound_stats(&ilp, &self.listen_addr, this_bloom, "********* RESPONSE PACKET FOUND *********");
                            debug!("********* RESPONSE PACKET FOUND *********");
                            let wp = WirePacket::new(this_link.reply_to(), nw);
                            let ilp = InterLinkPacket::new(this_link.clone(), wp);
                            r2c_tx.send(ilp).unwrap();
                            return Ok(());
                        }
                        None => {
                            debug!("********* NO   RESPONSE   FOUND *********");
                            let mut is_forwarded = false;
                            let mut broadcast = Vec::new();
                            this_bloom.create_pending_request(&hbfi);
                            //inbound_stats(&ilp, &self.listen_addr, this_bloom, "Inserting pending request");
                            for (that_link, that_bloom) in blooms.iter_mut() {
                                if that_link.id() == this_link_id {
                                    continue;
                                }
                                if that_bloom.contains_forwarded_request(&hbfi) > 51 {
                                    //outbound_stats(&ilp, &self.listen_addr, that_bloom, "Don't send request upstream again");
                                    continue;
                                }
                                if that_bloom.contains_pending_request(&hbfi) > 51 {
                                    //outbound_stats(&ilp, &self.listen_addr, that_bloom, "Don't send request downstream");
                                    continue;
                                }
                                if that_bloom.contains_forwarding_hint(&hbfi) > 90 {
                                    that_bloom.create_forwarded_request(&hbfi);
                                    //outbound_stats(&ilp, &self.listen_addr, that_bloom, "Sending request downstream based on forwarding hint");
                                    r2c_tx
                                        .send(ilp.change_destination(that_link.clone()))
                                        .unwrap();
                                    is_forwarded = true;
                                    continue;
                                }
                                broadcast.push(that_link.clone())
                            }
                            if !is_forwarded {
                                for broadcast_link in broadcast {
                                    if let Some(burst_bloom) =
                                        blooms.get_mut(&broadcast_link.clone())
                                    {
                                        burst_bloom.create_forwarded_request(&hbfi);
                                        //outbound_stats(&ilp, &self.listen_addr, burst_link, "Bursting on face");
                                        r2c_tx
                                            .send(ilp.change_destination(broadcast_link.clone()))
                                            .unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
                NarrowWaist::Response { hbfi, .. } => {
                    if this_bloom.contains_forwarded_request(&hbfi) > 15 {
                        response_store.insert(hbfi.try_to_vec()?, nw.clone().try_to_vec()?)?;
                        if this_bloom.forwarding_hint_decoherence() > 80 {
                            this_bloom.partially_forget_forwarding_hint();
                        }
                        this_bloom.delete_forwarded_request(&hbfi);
                        this_bloom.create_forwarding_hint(&hbfi);
                        for (that_link, that_bloom) in blooms.iter_mut() {
                            if that_link.id() == this_link_id {
                                continue;
                            }
                            if that_bloom.contains_pending_request(&hbfi) > 50 {
                                //outbound_stats(&ilp, &self.listen_addr, that_bloom, "Send response upstream");
                                debug!("********* RESPONSE DOWNSTREAM *********");
                                r2c_tx
                                    .send(ilp.change_destination(that_link.clone()))
                                    .unwrap();
                            }
                        }
                    }
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    }
}
/*
fn inbound_stats(packet: &InterLinkPacket, router_id: &ReplyTo, face: &Link, message: &str) {
    let print = format!(
        "INBOUND PACKET for {:?}\n\t{:?}\n\tFrom {:?} => To {:?}\n\t{}\n\t\t{}",
        router_id,
        packet,
        face.id(),
        router_id,
        face_stats(face, packet),
        message,
    );
    trace!("{}", print);
}

fn outbound_stats(packet: &InterLinkPacket, router_id: &ReplyTo, face: &Link, message: &str) {
    let print = format!(
        "OUTBOUND PACKET for {:?}\n\t{:?}\n\tFrom {:?} => To {:?}\n\t{}\n\t\t{}",
        router_id,
        packet,
        router_id,
        face.id(),
        face_stats(face, packet),
        message,
    );
    trace!("{}", print);
}

#[allow(dead_code)]
fn all_links_stats(links: &HashMap<ReplyTo, Link>, packet: &InterLinkPacket, message: &str) {
    let mut s: String = message.to_string();
    for (_link, face) in links {
        s.push_str(&format!("\n\t"));
        s.push_str(&face_stats(face, packet));
    }
    trace!("{}",s);
}

fn face_stats(face: &Link, packet: &InterLinkPacket) -> String {
    let hbfi: HBFI = match packet.payload() {
        NarrowWaist::Request{hbfi} => hbfi,
        NarrowWaist::Response{hbfi,..} => hbfi,
    };
    format!(
    "[pr{0: <3}d{1: <3}fr{2: <3}d{3: <3}fh{4: <3}d{5: <0}] faceid {6:?} hbfi {7:?}",
        face.contains_pending_request(&hbfi),
        face.pending_request_decoherence(),
        face.contains_forwarded_request(&hbfi),
        face.forwarded_request_decoherence(),
        face.contains_forwarding_hint(&hbfi),
        face.forwarding_hint_decoherence(),
        face.id(),
        hbfi)
}

*/
