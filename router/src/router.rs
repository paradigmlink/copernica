use packets::{Interest, Data};
use faces::{Face, Mock};
use crate::content_store::{ContentStore};

#[derive(Clone)]
pub struct Router {
    faces: Vec<Mock>,
    cs:  ContentStore,
    is_running: bool,
}

/*
    -> interest goes out on a face, and is written in the face's bread-crumb-sdr
        - checks the cs for data
            - if found
                - the interest is killed and data is returned via the sending interface
            - if cs is empty
                - interest index is written into incoming face breadcrumb-sdr
                - then search all other faces' forwarding sdr
                    - faces who's forwarding hint hits of a certain percentage AND miss on the faces' pending interest gets the interest forwarded
                        - forwarded interests are then inserted into the face's pending interest sdr (so that we don't forward the same interest again)
                    - if no face contains a forwarding interest hit then the interest is broadcast on all faces except the incoming face

    <- data comes in on a face
        - checks the face's pending interest sdr
            - if there's a miss on the pisdr then the data is dropped
            - if there's a hit on the pisdr
                - then the pisdr entry is removed on that face
                - forwarding-hint sdr insert - so future interests can be forwarded
                - then scan all other faces' breadcrumb-sdr
                    - if there's a hit then forward the data on that face
                        - remove the breadcrumb sdr data entry
                    - if there's a miss then drop the data
                - stick the data in the cs

    forwarding-sdr: is constructed from an LRU (index), which regenerates after a percentage of on bits reaches a threshold.
        this way the sdr maintains a high degree of data it's aware of and can adapt when downstream changes.

    pending-sdr: is constructed from an LRU (index) and is regenerated after the sdr reaches a critical threshold of on bits.
        if a data returns and the sdr has been regenerated then check other faces for breadcrumbs, if none, drop the data.
        the pending interest sdr is so the router will not forward the same interest again on that face.

    breadcrumb-sdr: is constructed from an LRU(index) and is regenerated after the sdr reaches a critical threshold of on bits.
        An index is written immediately into the LRU and sdr when an interest comes in. If the interest is not satisfied after a certain
        threshold the sdr is regenerated. If data is returned the breadcrumb-sdr entry is removed from the LRU and sdr.

*/

impl Router {
    pub fn new() -> Self {
        Router {
            faces: Vec::new(),
            cs:  ContentStore::new(),
            is_running: false,
        }
    }

    pub fn add_face(&mut self, face: Mock) {
        self.faces.push(face);
    }

    pub fn run(&mut self) {
        self.is_running = true;
        loop {
            for i in 0 .. self.faces.len() {
                let (face, other_faces) = self.faces.split_one_mut(i);
                match face.receive_upstream_interest() {
                    Some(i) => {
                        match self.cs.has_data(i.clone()) {
                            Some(d) => {
                                face.send_data_upstream(d);
                                continue
                            },
                            None => {
                                face.create_breadcrumb_trail(i.clone());
                                let mut is_forwarded = false;
                                let mut burst_faces: Vec<&mut Mock> = Vec::new();
                                for maybe_forward_face in other_faces {
                                    if maybe_forward_face.contains_pending_interest(i.clone()) > 90 &&
                                       maybe_forward_face.contains_forwarding_hint(i.clone())  > 10 {
                                        maybe_forward_face.create_pending_interest(i.clone());
                                        maybe_forward_face.send_interest_downstream(i.clone());
                                        is_forwarded = true;
                                        continue
                                    } else {
                                        burst_faces.push(maybe_forward_face);
                                    }
                                }
                                if is_forwarded == false {
                                    for burst_face in burst_faces {
                                        burst_face.create_pending_interest(i.clone());
                                        burst_face.send_interest_downstream(i.clone());
                                        continue
                                    }
                                }
                                continue
                            }
                        }
                    },
                    None => { continue },
                }
            }
            if self.is_running == false { break }
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
    use crossbeam_channel::{unbounded};


    #[test]
    fn test_cs() {
        let mut r1 = Router::new();
        let f1: Mock = Face::new();
        let f2: Mock = Face::new();
        let f3: Mock = Face::new();
        let f4: Mock = Face::new();
        let i1 = Interest::new("interest 1".to_string());
        let i2 = Interest::new("interest 2".to_string());
        f1.send_interest_downstream(i1);
        f2.send_interest_downstream(i2);
        r1.add_face(&f1);
        r1.add_face(&f2);
        r1.add_face(&f3);
        r1.add_face(&f4);
        r1.run();
        r1.stop();
        println!("data out {:?}", f1.receive_downstream_data());
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
