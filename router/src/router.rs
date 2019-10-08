use packets::{Interest, Data};
use faces::{Face, Mock};
use crate::content_store::{ContentStore};
use crate::pending_interest_table::{PendingInterestTable};
use crate::forwarding_information_base::{ForwardingInformationBase};
//use crossbeam_utils::{thread::scope};
use std::thread;

#[derive(Clone)]
pub struct Router<'a> {
    faces: Vec<&'a dyn Face>,
    cs:  ContentStore,
    pit: PendingInterestTable,
    fib: ForwardingInformationBase,
    is_running: bool,
}

/*
    1) interest comes in on a face, and is written in the face's bread-crumb-sdr
    2) the router then looks up *other* faces' forwarding-sdr
    3) if another face's f-sdr matches, the interest is forwarded to that face, index is written to face's pending interest table
    4) data comes back, the face checks to see if the interest index matches in the pi-sdr, if it does the pi-sdr entry is removed
    and added to the forwarding-sdr.
    5) now take the data and check all the other faces' bread-crumb-sdr, then return the data to every face that has a hit. Removing
    the entry from the bread-crumb-sdr of each face.
    6) the data is then added to the content store.
    7) each forwarding-sdr is regenerated after a certain percentage of bits are flipped to ensure sparsity.

    forwarding-sdr: is constructed from an LRU (index), which regenerates after a percentage of on bits reaches a threshold.
        this way the sdr maintains a high degree of data it's aware of and can adapt when downstream changes.

    pending-sdr: is constructed from an LRU (index) and is regenerated after the sdr reaches a critical threshold of on bits.
        if a data returns and the sdr has been regenerated then check other faces for breadcrumbs, if none, drop the data.
        the pending interest sdr is so the router will not forward the same interest again on that face.

    breadcrumb-sdr: is constructed from an LRU(index) and is regenerated after the sdr reaches a critical threshold of on bits.
        An index is written immediately into the LRU and sdr when an interest comes in. If the interest is not satisfied after a certain
        threshold the sdr is regenerated. If data is returned the breadcrumb-sdr entry is removed from the LRU and sdr.

*/

impl<'a> Router<'a> {
    pub fn new() -> Self {
        Router {
            faces: Vec::new(),
            cs:  ContentStore::new(),
            pit: PendingInterestTable::new(),
            fib: ForwardingInformationBase::new(),
            is_running: false,
        }
    }

    pub fn add_face(&mut self, face: &'a dyn Face) {
        self.faces.push(face);
    }

    pub fn run(&mut self) {
        // add loop later
        self.is_running = true;
        for face in self.faces.iter() {
            match face.interest_out() {
                Some(i) => {
                    match self.cs.has_data(i) {
                        Some(d) => {
                            face.data_in(d);
                        },
                        None => { continue }
                    }
                },
                None => { },
            }
        }
    }

    pub fn stop(mut self) {
        self.is_running = false;
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
        f1.interest_in(i1);
        f2.interest_in(i2);
        r1.add_face(&f1);
        r1.add_face(&f2);
        r1.add_face(&f3);
        r1.add_face(&f4);
        r1.run();
        r1.stop();
        println!("data out {:?}", f1.data_out());
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
