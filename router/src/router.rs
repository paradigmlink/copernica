use {
    crossbeam_channel::{unbounded},
    packets::{Packet},
    crate::content_store::{ContentStore},
    faces::{Face, Executor, Spawner, new_spawner_and_executor},
};

#[derive(Clone)]
pub struct Router {
    faces: Vec<Box<dyn Face>>,
    cs:  ContentStore,
    is_running: bool,
}


impl Router {
    pub fn new() -> Self {
        Router {
            faces: Vec::new(),
            cs:  ContentStore::new(),
            is_running: false,
        }
    }

    pub fn add_face(&mut self, face: Box<dyn Face>) {
        self.faces.push(face);
    }

    pub async fn run(&mut self) {
        self.is_running = true;

        let (spawner, executor) = new_spawner_and_executor();
        let (packet_sender, packet_receiver) = unbounded();
        for face in self.faces.iter_mut() {
            face.receive_upstream_interest_or_downstream_data(spawner.clone(), packet_sender.clone());
        }
        std::thread::spawn(move || { executor.run() });
        loop {
            let packet = packet_receiver.recv().unwrap();
            let (this_face, other_faces) = self.faces.split_one_mut(0);
            match packet.clone() {
                // Interest Downstream
                Packet::Interest { sdri: sdri } => {
                    match self.cs.has_data(&sdri) {
                        Some(data) => {
                            this_face.send_data_upstream(packet);
                        },
                        None => {
                            let mut is_forwarded = false;
                            let mut optimistic_burst_faces = Vec::new();
                            for that_face in other_faces {
                                if that_face.contains_pending_interest(&sdri) > 90 &&
                                   that_face.contains_forwarding_hint(&sdri) > 10 {
                                    that_face.create_pending_interest(&sdri);
                                    that_face.send_interest_downstream(packet.clone());
                                    is_forwarded = true;
                                } else {
                                    if is_forwarded == false { optimistic_burst_faces.push(that_face); }
                                }
                            }
                            if is_forwarded == false {
                                for burst_face in optimistic_burst_faces {
                                //println!("face pi : {:?}", burst_face.print_pi());
                                    burst_face.create_pending_interest(&sdri);
                                //println!("face pi : {:?}", burst_face.print_pi());
                                    burst_face.send_interest_downstream(packet.clone());
                                }
                            }
                        },
                    }
                },
                // Data Upstream
                Packet::Data { sdri: sdri } => {
                    if this_face.contains_pending_interest(&sdri) > 15 {
                        this_face.delete_pending_interest(&sdri);
                        //@Optimisation: check on every return? maybe periodically check the forwarding hint?
                        if this_face.forwarding_hint_decoherence() > 80 {
                            this_face.partially_forget_forwarding_hints();
                        }
                        this_face.create_forwarding_hint(&sdri);
                        for that_face in other_faces {
                            if that_face.contains_pending_interest(&sdri) > 50 {
                                that_face.delete_pending_interest(&sdri);
                                that_face.send_data_upstream(packet.clone());
                            }
                        }
                    }
                },
            };
        }
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
