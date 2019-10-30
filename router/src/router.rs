use {
    crossbeam_channel::{unbounded},
    packets::{Packet},
    content_store::{ContentStore, InMemory},
    faces::{Face},
    futures::executor::ThreadPool,
    log::{trace},
};

#[derive(Clone)]
pub struct Router {
    faces: Vec<Box<dyn Face>>,
    cs:  Vec<Box<dyn ContentStore>>,
    spawner: ThreadPool,
    is_running: bool,
}

impl Router {
    pub fn new(spawner: ThreadPool) -> Self {
        Router {
            faces: Vec::new(),
            cs:  vec![InMemory::new()],
            spawner: spawner,
            is_running: false,
        }
    }

    pub fn add_content_store(&mut self, cs: Box<dyn ContentStore>) {
        self.cs.push(cs);
    }

    pub fn insert_into_cs(&mut self, response: Packet) {
        trace!("[SETUP] adding named data");
        self.cs[0].put_data(response);
    }

    pub fn add_face(&mut self, mut face: Box<dyn Face>) {
        face.set_id(self.faces.len());
        trace!("[SETUP] adding face with id {}", face.get_id());
        self.faces.push(face);
    }

    pub async fn run(&mut self) {
        self.is_running = true;
        let (packet_sender, packet_receiver) = unbounded();
        for face in self.faces.iter_mut() {
            face.receive_upstream_request_or_downstream_response(self.spawner.clone(), packet_sender.clone());
        }
        loop {
            let (face_id, packet) = packet_receiver.recv().unwrap();
            let (this_face, other_faces) = self.faces.split_one_mut(face_id);
            match packet.clone() {
                // Request Downstream
                Packet::Request { sdri } => {
                    // go through all content stores looking for data
                    let mut data: Option<Packet> = None;
                    // self.cs[0] is essentially a read write content store used for forwarding
                    // the other stores, if there should be any are for users or people that want to make other sources of data available
                    for cs in self.cs.iter() {
                        data = cs.has_data(&sdri);
                        if data != None {
                            break
                        }
                    }
                    match data {
                        Some(data) => {
                            // we can match a response to the request, send it back immediately
                            trace!("[RESUP] found data in content store");
                            this_face.send_response_upstream(data);
                        },
                        None => {
                            trace!("[REQDN] no data in content stores");
                            // there is no matched response so we need to forward the request
                            let mut is_forwarded = false;
                            // optimistic_burst_faces are in case we have no forwarding hints to use.
                            let mut optimistic_burst_faces = Vec::new();
                            // this face must indicate it wants matching responses upstream'ed on it
                            this_face.create_pending_request(&sdri); // <- 1 of 2 purposes of pending requests
                            trace!("[REQDN {}] left breadcrumb ", face_stats("IN", this_face, &sdri));
                            for that_face in other_faces {
                                trace!("[REQDN] checking which faces to forward request on");
                                // Pending requests serve two purposes
                                // 1 leave a breadcrumb trail to return responses along (see immediately above)
                                // 2 stop us from forwarding a request on a face we have already forwarded a request on
                                // We are now checking the latter now.

                                // If that_face forwarding hint is high it means a previous request has been satisfied and thus we are likely to get satisfied again so we should forward it on that_face
                                if that_face.contains_forwarding_hint(&sdri) > 90 {
                                    trace!("[REQDN {}] is a good face to forward on",  face_stats("OUT", that_face, &sdri));
                                    // but even if we have the slightest hint (greater than 10%) we have already forwarded it on this face, we shouldn't forward it again. This system must be in flow balance.
                                    if that_face.contains_pending_request(&sdri) > 10 {
                                        trace!("[REQDN {}] already forwarded on this face ", face_stats("OUT", that_face, &sdri));
                                        // we won't set is_forwarded to true nor will we add this face to the burst faces. We will be good citizens and not spam someone who has already received this request.
                                        trace!("[REQDN] not adding face to burst faces");
                                        continue
                                    }
                                    // create a pending request indicating not to forward further requests on this face
                                    trace!("[REQDN] creating pending request and forwarding");
                                    trace!("[REQDN {}] creating pending request", face_stats("OUT", that_face, &sdri));
                                    that_face.create_pending_request(&sdri);
                                    trace!("[REQDN {}] sending request downstream", face_stats("OUT", that_face, &sdri));
                                    // downstream the request
                                    that_face.send_request_downstream(packet.clone());
                                    // is_forwarded is set to true meaning we don't need to flood the request on all that_faces
                                    is_forwarded = true;
                                }
                                if is_forwarded == false {
                                    trace!("[REQDN {}] adding burst face", face_stats("OUT", that_face, &sdri));
                                    optimistic_burst_faces.push(that_face)
                                };
                            }
                            if is_forwarded == false {
                                    trace!("[REQDN] bursting");
                                // we haven't forwarded on any face, so likely we haven't seen this request before, and have no idea where a response is. We will now flood each valid face with the request hoping that a face will be able to find it.
                                for burst_face in optimistic_burst_faces {
                                    // again we set the pending request so that further requests are not forwarded on this face
                                    trace!("[REQDN {}] creating pr", face_stats("BURST", burst_face, &sdri));
                                    burst_face.create_pending_request(&sdri);
                                    trace!("[REQDN {}] bursting on face", face_stats("BURST", burst_face, &sdri));
                                    burst_face.send_request_downstream(packet.clone());
                                }
                            }
                        },
                    }
                },
                // Response Upstream phase
                Packet::Response { sdri, data: _data } => {
                    // should this face contain a small probability of a match (greater than 15%) then we can assume this face was interested in this request and was previously forwarded on it.
                    trace!("[RESUP {}] received a resonse", face_stats("IN", this_face, &sdri));
                    if this_face.contains_pending_request(&sdri) > 15 {
                        trace!("[RESUP {}] contains pr", face_stats("IN", this_face, &sdri));
                        // we can delete this pending request to make room for other pending requests, note, there are over lapping bits with other requests that are being removed. This is how the decoherence happens.
                        this_face.delete_pending_request(&sdri);
                        //@Optimisation: check on every return? maybe periodically check the forwarding hint?
                        // if there is too much decoherence then we need to forget, by this I mean randomly remove bits. Existing successfully forwarded requests will decohere greatly. It's like going to sleep and waking up refreshed. Note tuning is required.
                        if this_face.forwarding_hint_decoherence() > 80 {
                            trace!("[RESUP {}] high fh decoherence", face_stats("IN", this_face, &sdri));
                            this_face.partially_forget_forwarding_hints();
                            trace!("[RESUP {}] cleaned fh decoherence", face_stats("IN", this_face, &sdri));
                        }
                        // we should now inform future upstreaming requests that this face is good as it successfully returns responses.
                        trace!("[RESUP {}] about to create fh", face_stats("IN", this_face, &sdri));
                        this_face.create_forwarding_hint(&sdri);
                        trace!("[RESUP {}] created fh", face_stats("IN", this_face, &sdri));
                        // let's insert this data into our in memory content store used for forwarding and returning responses.
                        trace!("[RESUP] inserting response into content store");
                        self.cs[0].put_data(packet.clone());
                        // let us now go over the breadcrumb trail dropped and see which other faces are interested in forwarding a response on thier faces
                        trace!("[RESUP] checking faces to forward response on");
                        for that_face in other_faces {
                            trace!("[RESUP {}] trying face", face_stats("OUT", that_face, &sdri));
                            // should a face express a medium level interest in a response then we should return a response on that face.
                            if that_face.contains_pending_request(&sdri) > 50 {
                                trace!("[RESUP {}] contains pr", face_stats("OUT", that_face, &sdri));
                                // we have satisfied the request and thus we can show it the door to oblivion
                                that_face.delete_pending_request(&sdri);
                                // send the RESUP
                                trace!("[RESUP {}] sending on that face", face_stats("OUT", that_face, &sdri));
                                that_face.send_response_upstream(packet.clone());
                            }
                        }
                    }
                },
            };
        } // loop end
    }

    pub fn stop(mut self) {
        self.is_running = false;
    }

}

fn face_stats(direction: &str, face: &mut Box<dyn Face>, sdri: &Vec<Vec<u16>>) -> String {
    format!(
    "{0: <5} face{1: <1} pr{2: <3} prd{3: <3} fh{4: <3} fhd{5: <0}",
        direction,
        face.get_id(),
        face.contains_pending_request(&sdri),
        face.pending_request_decoherence(),
        face.contains_forwarding_hint(&sdri),
        face.forwarding_hint_decoherence())
}


type ImplIteratorMut<'a, Item> =
    ::std::iter::Chain<
        ::std::slice::IterMut<'a, Item>,
        ::std::slice::IterMut<'a, Item>,
    >
;
trait SplitOneMut {
    type Item;

    fn split_one_mut (
        self: &'_ mut Self,
        i: usize,
    ) -> (&'_ mut Self::Item, ImplIteratorMut<'_, Self::Item>);
}

impl<T> SplitOneMut for [T] {
    type Item = T;

    fn split_one_mut (
        self: &'_ mut Self,
        i: usize,
    ) -> (&'_ mut Self::Item, ImplIteratorMut<'_, Self::Item>)
    {
        let (prev, current_and_end) = self.split_at_mut(i);
        let (current, end) = current_and_end.split_at_mut(1);
        (
            &mut current[0],
            prev.iter_mut().chain(end),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use packets::{mk_interest, mk_data, Packet};
    use faces::{Face, Tcp, Uds};
    use std::thread;

    #[test]
    fn test_cs() {
        let mut f1 = Uds::new();
        let mut f2 = Uds::new();
        let f3 = Uds::new();
        let f4 = Uds::new();
        let i1 = mk_interest("interest 1".to_string());
        let i2 = mk_interest("interest 2".to_string());
        f1.send_interest_downstream(i1);
        f2.send_interest_downstream(i2);
        thread::spawn(move || {
            let mut r1 = Router::new();
            r1.add_face(f1);
            r1.add_face(f2);
            r1.add_face(f3);
            r1.add_face(f4);
            r1.run();
        });

//        r1.stop();
    }
/*
    #[test]
    fn setup_router_and_ensure_faces_still_operate_does_not_pass_ownership_into_router() {
        let f1: Mock = Face::new();
        let mut router = Router::new();
        router.add_face(&f1);
        let interest = Interest::new("interest".to_string());
        f1.interest_in(interest.clone());
        assert_eq!(interest, f1.interest_poll().unwrap());
    }

    #[test]
    fn test_throughput() {
        let f1: Mock = Face::new();
        let f2: Mock = Face::new();
        let f3: Mock = Face::new();
        let f4: Mock = Face::new();
        let mut r1 = Router::new();
        let mut r2 = Router::new();
        r1.add_face(&f1);
        r1.add_face(&f2);
        r2.add_face(&f3);
        r2.add_face(&f4);
        let interest = Interest::new("interest".to_string());
        let (i_in, i_out) = unbounded();
        f1.interest_in(interest.clone());
        r1.run();
        i_in.send(f2.interest_poll()).unwrap();
        f3.interest_in(i_out.recv().unwrap().unwrap());
        r2.run();
        assert_eq!(interest, f4.interest_poll().unwrap());

        r1.stop();
        r2.stop();
        // i -> f1 r1 f2 -> f3 r2 f4
    }

    #[test]
    fn batch_feed_interests() {
        let is: Vec<Interest> = vec![Interest::new("1".to_string()), Interest::new("2".to_string()), Interest::new("3".to_string())];
        let f1: Mock = Face::new();
        let f2: Mock = Face::new();
        let mut r1 = Router::new();
        r1.add_face(&f1);
        r1.add_face(&f2);
        for i in &is {
            f1.interest_in(i.clone());
        }
        r1.run();
        let mut ois: Vec<Interest> = Vec::new();
        while let Some(i) = f2.interest_poll() {
            ois.push(i);
        }
        assert_eq!(is, ois);
    }
*/
}
