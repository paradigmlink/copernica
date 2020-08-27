use {
    crate::{
        link::{Link},
        channel::{LinkId},
        packets::{
            TransportPacket, NarrowWaist
        },
        //hbfi::{HBFI},
        borsh::{BorshDeserialize, BorshSerialize},
    },
    anyhow::{Result},
    //log::{trace},
    crossbeam_channel::{Sender, Receiver},
    std::{
        collections::{HashMap},
    },
};

#[derive(Clone)]
pub struct Router {
}

impl Router {
    pub fn handle_packet(
        link_id: &LinkId
        , transport_packet: &TransportPacket
        , outbound_tx: Sender<(LinkId, TransportPacket)>
        , response_store: sled::Db
        , links: &mut HashMap::<LinkId, (Link, (Sender<(LinkId, TransportPacket)>, Receiver<(LinkId, TransportPacket)>))>
        ) -> Result<()> {
        let narrow_waist_packet: NarrowWaist = transport_packet.payload();
        if let Some((this_link, _)) = links.get_mut(&link_id) {
            match narrow_waist_packet.clone() {
                NarrowWaist::Request { hbfi } => {
                    match response_store.get(&hbfi.try_to_vec()?)? {
                        Some(response) => {
                            let narrow_waist = NarrowWaist::try_from_slice(&response)?;
                            //outbound_stats(&transport_packet, &self.listen_addr, this_link, "********* RESPONSE PACKET FOUND *********");
                            let tp = TransportPacket::new(transport_packet.reply_to(), narrow_waist);
                            outbound_tx.send((link_id.clone(), tp)).unwrap();
                            return Ok(())
                        },
                        None => {
                            let mut is_forwarded = false;
                            let mut broadcast = Vec::new();
                            this_link.create_pending_request(&hbfi);
                            //inbound_stats(&transport_packet, &self.listen_addr, this_link, "Inserting pending request");
                            for (that_link_id, (that_link, _)) in links.iter_mut() {
                                if *that_link_id == *link_id {
                                    continue
                                }
                                if that_link.contains_forwarded_request(&hbfi) > 51 {
                                    //outbound_stats(&transport_packet, &self.listen_addr, that_link, "Don't send request upstream again");
                                    continue
                                }
                                if that_link.contains_pending_request(&hbfi)   > 51 {
                                    //outbound_stats(&transport_packet, &self.listen_addr, that_link, "Don't send request downstream");
                                    continue
                                }
                                if that_link.contains_forwarding_hint(&hbfi)   > 90 {
                                    that_link.create_forwarded_request(&hbfi);
                                    //outbound_stats(&transport_packet, &self.listen_addr, that_link, "Sending request downstream based on forwarding hint");
                                    outbound_tx.send((that_link_id.clone(), transport_packet.clone())).unwrap();
                                    is_forwarded = true;
                                    continue
                                }
                                broadcast.push(that_link_id.clone())

                            }
                            if !is_forwarded {
                                for broadcast_link_id in broadcast {
                                    if let Some((burst_link, _)) = links.get_mut(&broadcast_link_id.clone()) {
                                        burst_link.create_forwarded_request(&hbfi);
                                        //outbound_stats(&transport_packet, &self.listen_addr, burst_link, "Bursting on face");
                                        outbound_tx.send((broadcast_link_id.clone(), transport_packet.clone())).unwrap();
                                    }
                                }
                            }
                        },
                    }
                },
                NarrowWaist::Response { hbfi, .. } => {
                    if this_link.contains_forwarded_request(&hbfi) > 15 {
                        response_store.insert(hbfi.try_to_vec()?, narrow_waist_packet.clone().try_to_vec()?)?;
                        if this_link.forwarding_hint_decoherence() > 80 {
                            this_link.partially_forget_forwarding_hint();
                        }
                        /*
                        if response_store.complete(&hbfi) {
                            this_link.delete_forwarded_request(&hbfi);
                            this_link.create_forwarding_hint(&hbfi);
                        }
                        */
                        for (that_link_id, (that_link, _)) in links.iter_mut() {
                            if *that_link_id == *link_id { continue }
                            if that_link.contains_pending_request(&hbfi) > 50 {
                                //outbound_stats(&transport_packet, &self.listen_addr, that_link, "Send response upstream");
                                outbound_tx.send((that_link_id.clone(), transport_packet.clone())).unwrap();
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
fn inbound_stats(packet: &TransportPacket, router_id: &ReplyTo, face: &Link, message: &str) {
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

fn outbound_stats(packet: &TransportPacket, router_id: &ReplyTo, face: &Link, message: &str) {
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
fn all_links_stats(links: &HashMap<ReplyTo, Link>, packet: &TransportPacket, message: &str) {
    let mut s: String = message.to_string();
    for (_link_id, face) in links {
        s.push_str(&format!("\n\t"));
        s.push_str(&face_stats(face, packet));
    }
    trace!("{}",s);
}

fn face_stats(face: &Link, packet: &TransportPacket) -> String {
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
