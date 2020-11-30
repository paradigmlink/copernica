use crate::{time::Time, PrivateIdentity, PublicIdentity, Signature};
use bytes::{Buf, BufMut};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    fmt::{self, Display, Formatter},
    rc::Rc,
    str::FromStr,
};
use thiserror::Error;
use tokio_util::codec::{Encoder, Decoder};

#[derive(Debug, Error)]
pub enum EventBinaryError {
    #[error(
        "Invalid size, missing {missing} bytes, expecting from {} to {} bytes",
        Event::SIZE_MIN,
        Event::SIZE_MAX
    )]
    InvalidSize { missing: usize },

    #[error("Invalid event type Ox{value:04x}")]
    InvalidEventType { value: u16 },
}

#[derive(Debug, Error)]
pub enum PassportError {
    #[error("The operation was not authorized")]
    UnAuthorized,

    #[error("Cannot repudiate a repudiation event")]
    CannotRepudiateRepudiate,

    #[error("Cannot repudiate unknown event {event_id}")]
    CannotRepudiateUnknownEventId { event_id: EventId },

    #[error("The event does not contains a valid signature proof")]
    InvalidSignature,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Passport {
    events: Vec<Rc<Event>>,
    map: HashMap<EventId, Rc<Event>>,
    ids: HashSet<PublicIdentity>,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct EventId([u8; Self::SIZE]);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct EventNumber(u32);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum EventType {
    Initialization,
    Repudiation,
    Declaration,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Event {
    pub previous: EventId,
    pub number: EventNumber,
    pub time: Time,
    pub author: PublicIdentity,
    pub action: EventAction,

    proof: Vec<Signature>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Service(String);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Alias(String);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum EventAction {
    Initialization,
    Repudiation {
        event: EventId,
    },
    Declaration {
        with: PublicIdentity,
    },
}

impl Passport {
    fn empty() -> Self {
        Self {
            events: Vec::with_capacity(12),
            map: HashMap::with_capacity(12),
            ids: HashSet::with_capacity(8),
        }
    }

    pub fn prepare_next_event(&self, action: EventAction) -> Event {
        let mut event = Event::default();
        event.number = EventNumber(self.events.len() as u32 + 1);
        event.previous = self
            .events
            .iter()
            .last()
            .map(|prev| prev.id())
            .unwrap_or_default();
        event.action = action;
        event
    }

    pub fn new(id: &PrivateIdentity) -> Self {
        let mut s = Self::empty();

        let action = EventAction::Initialization;
        if let Err(error) = s.push_unchecked(id, action) {
            unreachable!(
                "The initialization of a Passport should not fail: {}",
                error
            )
        }

        s
    }

    pub fn new_with(event: Event) -> Result<Self, PassportError> {
        let mut s = Self::empty();

        s.load_event(event)?;

        Ok(s)
    }

    pub fn load_event(&mut self, event: Event) -> Result<(), PassportError> {
        let event_id = event.id();

        let expect_initialization = self.events.is_empty();

        if !expect_initialization && !self.check_identity(&event.author) {
            return Err(PassportError::UnAuthorized);
        }

        if !event.verify() {
            return Err(PassportError::InvalidSignature);
        }

        match &event.action {
            EventAction::Initialization => {
                self.ids.insert(event.author.clone());
            }
            EventAction::Declaration { with, .. } => {
                self.ids.insert(with.clone());
            }
            EventAction::Repudiation { event: event_id } => {
                if let Some(event) = self.map.get(event_id) {
                    match &event.action {
                        EventAction::Initialization => {
                            self.ids.remove(&event.author);
                        }
                        EventAction::Repudiation { .. } => {
                            return Err(PassportError::CannotRepudiateRepudiate)
                        }
                        EventAction::Declaration { with } => {
                            self.ids.remove(with);
                        }
                    }
                } else {
                    return Err(PassportError::CannotRepudiateUnknownEventId {
                        event_id: *event_id,
                    });
                }
            }
        }

        let event = Rc::new(event);
        self.events.push(event.clone());
        self.map.insert(event_id, event);
        Ok(())
    }

    pub fn check_identity(&self, id: &PublicIdentity) -> bool {
        self.ids.contains(id)
    }

    fn push_unchecked(
        &mut self,
        id: &PrivateIdentity,
        action: EventAction,
    ) -> Result<EventId, PassportError> {
        let mut event = Event::new();
        event.number = EventNumber(self.events.len() as u32 + 1);
        event.previous = self
            .events
            .iter()
            .last()
            .map(|prev| prev.id())
            .unwrap_or_default();
        event.action = action;
        event.force_self_sign(id);

        let event_id = event.id();

        match &event.action {
            EventAction::Initialization => {
                self.ids.insert(id.public_id());
            }
            EventAction::Declaration { with, .. } => {
                self.ids.insert(with.clone());
            }
            EventAction::Repudiation { event: event_id } => {
                if let Some(event) = self.map.get(event_id) {
                    match &event.action {
                        EventAction::Initialization => {
                            self.ids.remove(&event.author);
                        }
                        EventAction::Repudiation { .. } => {
                            return Err(PassportError::CannotRepudiateRepudiate)
                        }
                        EventAction::Declaration { with } => {
                            self.ids.remove(with);
                        }
                    }
                } else {
                    return Err(PassportError::CannotRepudiateUnknownEventId {
                        event_id: *event_id,
                    });
                }
            }
        }

        let event = Rc::new(event);
        self.events.push(event.clone());
        self.map.insert(event_id, event);
        Ok(event_id)
    }

    pub fn push(
        &mut self,
        id: &PrivateIdentity,
        action: EventAction,
    ) -> Result<EventId, PassportError> {
        if !self.check_identity(&id.public_id()) {
            Err(PassportError::UnAuthorized)
        } else {
            self.push_unchecked(id, action)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Event> {
        self.events.iter().map(|event| event.as_ref())
    }
}

impl EventType {
    const INITIALIZATION: u16 = 0x0001;
    const REPUDIATION: u16 = 0x0002;
    const DECLARATION: u16 = 0x0003;

    fn size(&self) -> usize {
        match self {
            Self::Initialization => 0,
            Self::Repudiation => EventId::SIZE,
            Self::Declaration => PublicIdentity::SIZE,
        }
    }

    fn size_with_proof(&self) -> usize {
        match self {
            Self::Initialization => 0,
            Self::Repudiation => EventId::SIZE,
            Self::Declaration => PublicIdentity::SIZE + Signature::SIZE,
        }
    }

    fn to_u16(&self) -> u16 {
        match self {
            Self::Initialization => Self::INITIALIZATION,
            Self::Repudiation => Self::REPUDIATION,
            Self::Declaration => Self::DECLARATION,
        }
    }
}

macro_rules! read_fixed_array {
    ($t:ty, $buf:ident) => {{
        let mut bytes = [0; <$t>::SIZE];
        $buf.copy_to_slice(&mut bytes);
        <$t>::from(bytes)
    }};
}

impl Event {
    const SIZE_COMMON: usize = 2 // 2 bytes of u16 for the event type
        + EventId::SIZE + EventNumber::SIZE + Time::SIZE + PublicIdentity::SIZE;
    pub const SIZE_MIN: usize = Self::SIZE_COMMON + EventAction::SIZE_MIN + Signature::SIZE;
    pub const SIZE_MAX: usize = Self::SIZE_COMMON + EventAction::SIZE_MAX + Signature::SIZE + Signature::SIZE;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(&self) -> EventId {
        let size = self.action.event_type().size() + Self::SIZE_COMMON;
        let mut bytes = [0; Self::SIZE_MAX];
        self.write_to_buf_mut(&mut bytes.as_mut()).unwrap();

        EventId::compute(&bytes[..size])
    }

    fn signing_data(&self) -> [u8; 32] {
        let size = self.action.event_type().size() + Self::SIZE_COMMON;
        let mut bytes = [0; Self::SIZE_MAX];
        let mut res = [0; 32];
        self.write_to_buf_mut(&mut bytes.as_mut()).unwrap();

        use cryptoxide::digest::Digest as _;
        let mut b = cryptoxide::blake2b::Blake2b::new(32);
        b.input(&bytes[..size]);
        b.result(&mut res);
        res
    }

    pub fn verify(&self) -> bool {
        let signing_data = self.signing_data();
        let mut proof = if let Some(signature) = self.proof.get(0) {
            self.author.verify_key().unwrap().verify(signature, &signing_data)
        } else {
            false
        };

        if let EventAction::Declaration { with } = &self.action {
            proof = proof && if let Some(signature) = self.proof.get(1) {
                with.verify_key().unwrap().verify(signature, signing_data)
            } else {
                false
            };
        };

        proof
    }

    pub fn force_self_sign(&mut self, id: &PrivateIdentity) {
        self.author = id.public_id();
        let proof = id.signing_key().sign(self.signing_data());
        if self.proof.is_empty() {
            self.proof.push(proof)
        } else {
            self.proof[0] = proof;
        }
    }

    pub fn force_signature(&mut self, id: &PrivateIdentity, index: usize) {
        let proof = id.signing_key().sign(self.signing_data());
        if self.proof.len() < index + 1 {
            self.proof.extend(std::iter::repeat(proof).take(index - self.proof.len() + 1));
        } else {
            self.proof[index] = proof;
        }
    }

    pub fn read_from_buf<B>(buf: &mut B) -> Result<Self, EventBinaryError>
    where
        B: Buf,
    {
        if buf.remaining() < Self::SIZE_MIN {
            return Err(EventBinaryError::InvalidSize { missing: Self::SIZE_MIN - buf.remaining()});
        }

        let action_type = EventType::try_from(buf.get_u16())?;
        let expected = Self::SIZE_MIN - 2 + action_type.size_with_proof();
        if buf.remaining() < expected {
            return Err(EventBinaryError::InvalidSize { missing: expected - buf.remaining() });
        }

        let previous = read_fixed_array!(EventId, buf);
        let number = EventNumber(buf.get_u32());
        let time = Time::from(buf.get_u64());
        let author = read_fixed_array!(PublicIdentity, buf);
        let mut number_expected_signature = 1;

        let action = match action_type {
            EventType::Initialization => EventAction::Initialization,
            EventType::Repudiation => {
                let event = read_fixed_array!(EventId, buf);
                EventAction::Repudiation { event }
            }
            EventType::Declaration => {
                let with = read_fixed_array!(PublicIdentity, buf);
                number_expected_signature += 1;

                EventAction::Declaration {
                    with,
                }
            }
        };

        let mut proof = Vec::with_capacity(number_expected_signature);
        for _ in 0..number_expected_signature {
            proof.push(read_fixed_array!(Signature, buf));
        }

        Ok(Self {
            previous,
            number,
            time,
            author,
            action,
            proof,
        })
    }

    fn size(&self) -> usize {
        let action_type = self.action.event_type();
        Self::SIZE_COMMON + action_type.size()
    }

    pub fn write_to_buf_mut<B>(&self, buf: &mut B) -> Result<(), EventBinaryError>
    where
        B: BufMut,
    {
        let needed_size = self.size();
        if buf.remaining_mut() < needed_size {
            return Err(EventBinaryError::InvalidSize { missing: needed_size - buf.remaining_mut() });
        }

        buf.put_u16(self.action.event_type().to_u16());
        buf.put_slice(&self.previous.0);
        buf.put_u32(self.number.0);
        buf.put_u64(*self.time);
        buf.put_slice(self.author.key().as_ref());
        buf.put_slice(self.author.chain_code().as_ref());
        match &self.action {
            EventAction::Initialization => (),
            EventAction::Repudiation { event } => buf.put_slice(&event.0),
            EventAction::Declaration {
                with,
            } => {
                buf.put_slice(with.key().as_ref());
                buf.put_slice(with.chain_code().as_ref());
            }
        }

        for proof in self.proof.iter() {
            buf.put_slice(proof.as_ref());
        }

        Ok(())
    }
}

impl EventNumber {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

impl EventId {
    pub const SIZE: usize = 16;

    const fn zero() -> Self {
        Self([0; Self::SIZE])
    }

    pub fn compute<B>(buf: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        use cryptoxide::digest::Digest as _;
        let mut b = cryptoxide::blake2b::Blake2b::new(Self::SIZE);
        b.input(buf.as_ref());
        let mut s = Self::zero();
        b.result(&mut s.0);
        s
    }
}

impl EventAction {
    pub const SIZE_MIN: usize = 0;
    pub const SIZE_MAX: usize = PublicIdentity::SIZE;

    pub fn event_type(&self) -> EventType {
        match self {
            Self::Initialization => EventType::Initialization,
            Self::Repudiation { .. } => EventType::Repudiation,
            Self::Declaration { .. } => EventType::Declaration,
        }
    }
}

/* Default ****************************************************************** */

impl Default for Event {
    fn default() -> Self {
        Self {
            previous: EventId::zero(),
            number: EventNumber(0),
            time: Time::now(),
            author: PublicIdentity::from([0; PublicIdentity::SIZE]),
            action: EventAction::Repudiation {
                event: EventId::zero(),
            },
            proof: Vec::with_capacity(2),
        }
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::zero()
    }
}

/* Codec ******************************************************************** */

pub struct EventCodec;

#[derive(Debug, Error)]
pub enum EventCodecError {
    #[error("Invalid event")]
    Invalid(#[from] #[source] EventBinaryError),

    #[error("I/O Error")]
    Io {
        #[from]
        #[source]
        source: std::io::Error,
    },
}


impl<'a> Encoder<&'a Event> for EventCodec {
    type Error = EventCodecError;
    fn encode(&mut self, item: &'a Event, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let remaining = dst.remaining_mut();
        let size = item.size();
        if remaining < size {
            dst.reserve(size - remaining);
        }
        Ok(item.write_to_buf_mut(dst)?)
    }
}

impl Decoder for EventCodec {
    type Item = Event;
    type Error = EventCodecError;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.remaining() < Event::SIZE_MIN {
            // if there is not enough in the source buffer we allocate what we know will be
            // missing to have at least one more valid block so the necessary allocation
            // is already done ahead of time
            src.reserve(Event::SIZE_MIN - src.len());
            Ok(None)
        } else {
            match Event::read_from_buf(src) {
                Ok(event) => Ok(Some(event)),
                Err(EventBinaryError::InvalidSize { missing }) => {
                    src.reserve(missing);
                    Ok(None)
                }
                Err(err) => Err(err.into()),
            }
        }
    }
}

/* Formatter **************************************************************** */

impl Display for EventId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        hex::encode(&self.0).fmt(f)
    }
}

impl Display for EventNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for EventId {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut eid = Self::zero();
        hex::decode_to_slice(s, &mut eid.0)?;
        Ok(eid)
    }
}

/* Conversion ************************************************************** */

impl From<EventNumber> for u32 {
    fn from(en: EventNumber) -> Self {
        en.0
    }
}

impl From<[u8; Self::SIZE]> for EventId {
    fn from(bytes: [u8; Self::SIZE]) -> Self {
        Self(bytes)
    }
}

impl From<EventId> for String {
    fn from(eid: EventId) -> Self {
        eid.to_string()
    }
}

impl TryFrom<String> for EventId {
    type Error = <Self as FromStr>::Err;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl<'a> TryFrom<&'a str> for EventId {
    type Error = <Self as FromStr>::Err;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<u16> for EventType {
    type Error = EventBinaryError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0u16 => Err(Self::Error::InvalidEventType { value }),
            EventType::INITIALIZATION => Ok(EventType::Initialization),
            EventType::REPUDIATION => Ok(EventType::Repudiation),
            EventType::DECLARATION => Ok(EventType::Declaration),
            0x0004..=u16::MAX => Err(Self::Error::InvalidEventType { value }),
        }
    }
}

impl AsRef<[u8]> for EventId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for EventId {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut bytes = [0; Self::SIZE];
            g.fill_bytes(&mut bytes);
            Self(bytes)
        }
    }

    impl Arbitrary for EventNumber {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Self(u32::arbitrary(g))
        }
    }

    impl Arbitrary for EventType {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            match u16::arbitrary(g) % 3 + 1 {
                0 => unreachable!(),
                Self::INITIALIZATION => Self::Initialization,
                Self::REPUDIATION => Self::Repudiation,
                Self::DECLARATION => Self::Declaration,
                0x0004..=u16::MAX => unreachable!(),
            }
        }
    }

    impl Arbitrary for EventAction {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            match EventType::arbitrary(g) {
                EventType::Initialization => Self::Initialization,
                EventType::Repudiation => Self::Repudiation {
                    event: EventId::arbitrary(g),
                },
                EventType::Declaration => Self::Declaration {
                    with: PublicIdentity::arbitrary(g),
                },
            }
        }
    }

    impl Arbitrary for Event {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let action = EventAction::arbitrary(g);
            let num_proof = match action.event_type() {
                EventType::Initialization => 1,
                EventType::Repudiation => 1,
                EventType::Declaration => 2,
            };

            Self {
                previous: EventId::arbitrary(g),
                number: EventNumber::arbitrary(g),
                time: Time::arbitrary(g),
                author: PublicIdentity::arbitrary(g),

                action,

                proof: std::iter::repeat_with(
                    || Signature::arbitrary(g)
                ).take(num_proof).collect(),
            }
        }
    }

    #[test]
    fn event_sizes() {
        dbg!(Event::SIZE_COMMON + Signature::SIZE + EventType::Initialization.size());
        dbg!(Event::SIZE_COMMON + Signature::SIZE + EventType::Repudiation.size());
        dbg!(Event::SIZE_COMMON + Signature::SIZE + EventType::Declaration.size() + Signature::SIZE);

        assert_eq!(Event::SIZE_COMMON, 94,);

        assert_eq!(Event::SIZE_MIN, Event::SIZE_COMMON + Signature::SIZE,);

        assert_eq!(Event::SIZE_COMMON + EventType::Initialization.size() + Signature::SIZE, Event::SIZE_MIN,);
        assert_eq!(Event::SIZE_COMMON + EventType::Repudiation.size() + Signature::SIZE, Event::SIZE_MIN + 16,);
        assert_eq!(Event::SIZE_COMMON + EventType::Declaration.size() + Signature::SIZE + Signature::SIZE, Event::SIZE_MIN + PublicIdentity::SIZE + Signature::SIZE);
        assert_eq!(Event::SIZE_COMMON + EventType::Declaration.size() + Signature::SIZE + Signature::SIZE, Event::SIZE_MAX);
    }

    #[quickcheck]
    fn event_pack_unpack(event: Event) -> bool {
        let mut buf = Vec::with_capacity(Event::SIZE_MAX);
        event.write_to_buf_mut(&mut buf).unwrap();
        let unpacked = Event::read_from_buf(&mut buf.as_slice()).unwrap();

        unpacked == event
    }

    #[quickcheck]
    fn event_sign_verify(event: Event, private_id: PrivateIdentity, with_id: PrivateIdentity) -> bool {
        let mut event = event;
        let extra = if let EventAction::Declaration { with } = &mut event.action {
            *with = with_id.public_id();
            true
        } else {
            false
        };
        event.force_self_sign(&private_id);
        if extra {
            event.force_signature(&with_id, 1);
        }
        event.verify()
    }
}
