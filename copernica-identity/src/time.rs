use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    fmt::{self, Debug, Display, Formatter},
    str::FromStr,
    time::SystemTime,
};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Time(u64);

impl Time {
    /// size of the `Time`
    ///
    /// ```
    /// # use keynesis::Time;
    /// assert_eq!(Time::SIZE, 8);
    /// ```
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// `Time` for `UNIX_EPOCH`
    ///
    pub const UNIX_EPOCH: Self = Self(0);

    /// get the current timestamp from the local system
    ///
    /// if the function is used outside of normal usage (i.e.
    /// if the local time is before UNIX_EPOCH) then the function
    /// has undefined behavior.
    pub fn now() -> Self {
        let now = SystemTime::now();
        let since_epoch = if let Ok(d) = now.duration_since(SystemTime::UNIX_EPOCH) {
            d.as_secs()
        } else {
            // this is impossible because the `SystemTime` is taken as `now` and
            // unless the users are playing silly with the local date and time
            // this is completely unreachable as the `now.duration_since(1/1/1970)`
            // will always successfully return something
            unsafe { std::hint::unreachable_unchecked() }
        };

        Self(since_epoch)
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for Time {
    type Err = <u64 as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl<'a> TryFrom<&'a str> for Time {
    type Error = <Self as FromStr>::Err;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl From<Time> for u64 {
    fn from(time: Time) -> Self {
        time.0
    }
}

impl From<u64> for Time {
    fn from(time: u64) -> Self {
        Self(time)
    }
}

impl std::ops::Deref for Time {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Time {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Self(u64::arbitrary(g))
        }
    }
}
