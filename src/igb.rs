use core::ptr::NonNull;

use log::debug;

pub struct Igb {}

impl Igb {
    pub fn new(bar0: NonNull<u8>) -> Self {
        unsafe {
            let ctr = 1 << 26;

            bar0.cast::<u32>().write_volatile(ctr);

            debug!("wait reset");

            loop {
                let ctr = bar0.cast::<u32>().read_volatile();
                if ctr & (1 << 26) == 0 {
                    break;
                }
            }

            debug!("reset");

            let status = bar0.cast::<u32>().read_volatile();
            let fd = status & 1;
            let lu = status & 1 << 1;
            let speed = status & 0b11 << 6;

            debug!("igb status: fd: {}, lu: {}, speed: {}", fd, lu, speed);
        }

        Self {}
    }
}
