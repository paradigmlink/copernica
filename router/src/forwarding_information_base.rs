
#[derive(Clone)]
pub struct ForwardingInformationBase {
    store: String,
}

impl ForwardingInformationBase {
    pub fn new() -> Self {
        ForwardingInformationBase { store: "".to_string() }
    }
}
