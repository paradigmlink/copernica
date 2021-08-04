use {
    crate::{HBFI},
    core::hash::{Hash, Hasher}
};
#[derive(Clone, Debug)]
pub struct HBFIOnlyKeys(pub HBFI);
impl Hash for HBFIOnlyKeys {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.request_pid.hash(state);
        self.0.response_pid.hash(state);
    }
}
impl PartialEq for HBFIOnlyKeys {
    fn eq(&self, other: &Self) -> bool {
        (self.0.request_pid == other.0.request_pid)
        && (self.0.response_pid == other.0.response_pid)
    }
}
impl Eq for HBFIOnlyKeys {}
