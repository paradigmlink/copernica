#[cfg(not(feature = "nightly"))]
use std::ptr;

pub trait Scrubbed {
    fn scrub(&mut self);
}

/// Perform a secure memset. This function is guaranteed not to be elided
/// or reordered.
///
/// # Performance consideration
///
/// On `nightly`, the function use a more efficient.
///
/// # Safety
///
/// The destination memory (`dst` to `dst+count`) must be properly allocated
/// and ready to use.
#[inline(never)]
pub unsafe fn memset(dst: *mut u8, val: u8, count: usize) {
    #[cfg(feature = "nightly")]
    {
        core::intrinsics::volatile_set_memory(dst, val, count);
    }

    #[cfg(not(feature = "nightly"))]
    {
        for i in 0..count {
            ptr::write_volatile(dst.add(i), val);
        }
    }
}

/// compare the equality of the 2 given arrays, constant in time
///
/// # Safety
///
/// Expecting to have both valid pointer and the count to fit in
/// both the allocated memories
#[inline(never)]
pub unsafe fn memeq(v1: *const u8, v2: *const u8, len: usize) -> bool {
    let mut sum = 0;

    for i in 0..len {
        let val1 = ptr::read_volatile(v1.add(i));
        let val2 = ptr::read_volatile(v2.add(i));

        let xor = val1 ^ val2;

        sum |= xor;
    }

    sum == 0
}

/// Constant time comparison
///
/// # Safety
///
/// Expecting to have both valid pointer and the count to fit in
/// both the allocated memories
#[inline(never)]
pub unsafe fn memcmp(v1: *const u8, v2: *const u8, len: usize) -> std::cmp::Ordering {
    let mut res = 0;
    for i in (0..len).rev() {
        let val1 = ptr::read_volatile(v1.add(i)) as i32;
        let val2 = ptr::read_volatile(v2.add(i)) as i32;
        let diff = val1 - val2;
        res = (res & (((diff - 1) & !diff) >> 8)) | diff;
    }
    let res = ((res - 1) >> 8) + (res >> 8) + 1;

    res.cmp(&0)
}

macro_rules! impl_scrubbed_primitive {
    ($t:ty) => {
        impl Scrubbed for $t {
            #[inline(never)]
            fn scrub(&mut self) {
                *self = 0;
            }
        }
    };
}

impl_scrubbed_primitive!(u8);
impl_scrubbed_primitive!(u16);
impl_scrubbed_primitive!(u32);
impl_scrubbed_primitive!(u64);
impl_scrubbed_primitive!(u128);
impl_scrubbed_primitive!(usize);
impl_scrubbed_primitive!(i8);
impl_scrubbed_primitive!(i16);
impl_scrubbed_primitive!(i32);
impl_scrubbed_primitive!(i64);
impl_scrubbed_primitive!(i128);
impl_scrubbed_primitive!(isize);

macro_rules! impl_scrubbed_array {
    ($t:ty) => {
        impl Scrubbed for $t {
            fn scrub(&mut self) {
                unsafe { memset(self.as_mut_ptr(), 0, self.len()) }
            }
        }
    };
}

impl_scrubbed_array!([u8; 2]);
impl_scrubbed_array!([u8; 4]);
impl_scrubbed_array!([u8; 8]);
impl_scrubbed_array!([u8; 16]);
impl_scrubbed_array!([u8; 24]);
impl_scrubbed_array!([u8; 32]);
impl_scrubbed_array!([u8; 40]);
impl_scrubbed_array!([u8; 48]);
impl_scrubbed_array!([u8; 56]);
impl_scrubbed_array!([u8; 64]);
impl_scrubbed_array!([u8; 128]);
impl_scrubbed_array!([u8; 256]);
impl_scrubbed_array!([u8; 512]);
impl_scrubbed_array!([u8]);
impl_scrubbed_array!(str);
