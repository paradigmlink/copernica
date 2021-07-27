use {
    crate::{HBFI},
    core::hash::{Hash, Hasher}
};
#[derive(Clone, Debug)]
pub struct HBFIExcludeFrame(pub HBFI);
impl Hash for HBFIExcludeFrame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.request_pid.hash(state);
        self.0.response_pid.hash(state);
        self.0.req.hash(state);
        self.0.res.hash(state);
        self.0.app.hash(state);
        self.0.m0d.hash(state);
        self.0.fun.hash(state);
        self.0.arg.hash(state)
    }
}
impl PartialEq for HBFIExcludeFrame {
    fn eq(&self, other: &Self) -> bool {
        (self.0.request_pid == other.0.request_pid)
        && (self.0.response_pid == other.0.response_pid)
        && (self.0.req == other.0.req)
        && (self.0.res == other.0.res)
        && (self.0.app == other.0.app)
        && (self.0.m0d == other.0.m0d)
        && (self.0.fun == other.0.fun)
        && (self.0.arg == other.0.arg)
    }
}
impl Eq for HBFIExcludeFrame {}
