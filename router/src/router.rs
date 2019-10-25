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
    is_running: bool,
}


impl Router {
    pub fn new() -> Self {
        let im = InMemory::new();
        Router {
            faces: Vec::new(),
            cs:  vec![im],
            is_running: false,
        }
    }

    pub fn add_content_store(&mut self, cs: Box<dyn ContentStore>) {
        self.cs.push(cs);
    }

    pub fn add_face(&mut self, face: Box<dyn Face>) {
        self.faces.push(face);
    }

    pub async fn run(&mut self, spawner: ThreadPool ) {
        self.is_running = true;
        let (packet_sender, packet_receiver) = unbounded();
        let mut face_id = 0;
        for face in self.faces.iter_mut() {
            face.receive_upstream_request_or_downstream_response(face_id, spawner.clone(), packet_sender.clone());
            face_id += 1;
        }
        loop {
            let (face_id, packet) = packet_receiver.recv().unwrap();
            let (this_face, other_faces) = self.faces.split_one_mut(face_id);
            match packet.clone() {
                // Request Downstream
                Packet::Request { sdri } => {
                    let mut data: Option<Packet> = None;
                    for cs in self.cs.iter() {
                        data = cs.has_data(&sdri);
                        if data != None {
                            trace!("found data: {:?}", data);
                            break
                        }
                    }
                    match data {
                        Some(_data) => {
                            this_face.send_response_upstream(packet);
                        },
                        None => {
                            let mut is_forwarded = false;
                            let mut optimistic_burst_faces = Vec::new();
                            for that_face in other_faces {
                                if that_face.contains_pending_request(&sdri) > 90 &&
                                   that_face.contains_forwarding_hint(&sdri) > 10 {
                                    that_face.create_pending_request(&sdri);
                                    that_face.send_request_downstream(packet.clone());
                                    is_forwarded = true;
                                } else {
                                    if is_forwarded == false { optimistic_burst_faces.push(that_face); }
                                }
                            }
                            if is_forwarded == false {
                                for burst_face in optimistic_burst_faces {
                                    burst_face.create_pending_request(&sdri);
                                    burst_face.send_request_downstream(packet.clone());
                                }
                            }
                        },
                    }
                },
                // Response Upstream
                Packet::Response { sdri, data: _data } => {
                    if this_face.contains_pending_request(&sdri) > 15 {
                        this_face.delete_pending_request(&sdri);
                        //@Optimisation: check on every return? maybe periodically check the forwarding hint?
                        if this_face.forwarding_hint_decoherence() > 80 {
                            this_face.partially_forget_forwarding_hints();
                        }
                        this_face.create_forwarding_hint(&sdri);
                        self.cs[0].put_data(packet.clone());
                        for that_face in other_faces {
                            if that_face.contains_pending_request(&sdri) > 50 {
                                that_face.delete_pending_request(&sdri);
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
