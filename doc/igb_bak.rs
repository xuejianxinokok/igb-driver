use core::ptr::NonNull;

use log::{debug, info};

pub struct Igb {}

impl Igb {
    pub fn new(bar0: NonNull<u8>) -> Self {
        unsafe {
            let mut ctrl = bar0.cast::<u32>().read_volatile();
            ctrl |= 1 << 26;
            bar0.cast::<u32>().write_volatile(ctrl);
            info!("start reset");
            loop {
                let ctrl = bar0.cast::<u32>().read_volatile();
                if ctrl & (1 << 26) == 0 {
                    break;
                }
            }

            debug!("reset");
            let status: u32 = bar0.add(8).cast::<u32>().read_volatile();
            let fd: u32 = status & 1;
            let lu: u32 = status & 1 << 1;
            let speed: u32 = (status & 0b11 << 6) >> 6;
            debug!("igb status:fd:{},lu:{}, speed: {:#b}", fd, lu, speed);

            Self {}
        }
    }
}
