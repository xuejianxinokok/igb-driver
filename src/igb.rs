use core::ptr::NonNull;

pub struct Igb {}

impl Igb {
    pub fn new(bar0: NonNull<u8>) -> Self {
        Self {}
    }
}
