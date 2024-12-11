use core::{convert::Infallible, ptr::NonNull};

use log::debug;

use crate::{
    descriptor::{AdvRxDesc, AdvTxDesc},
    err::IgbError,
    regs::{Reg, CTRL, RCTL},
    ring::{Ring, DEFAULT_RING_SIZE},
};

pub struct Igb {
    reg: Reg,
    machine: Machine,
    tx_ring: Ring<AdvTxDesc>,
    rx_ring: Ring<AdvRxDesc>,
}

impl Igb {
    pub fn new(bar0: NonNull<u8>) -> Result<Self, IgbError> {
        let reg = Reg::new(bar0);
        let tx_ring = Ring::new(reg, DEFAULT_RING_SIZE)?;
        let rx_ring = Ring::new(reg, DEFAULT_RING_SIZE)?;

        Ok(Self {
            reg,
            machine: Machine::Init,
            tx_ring,
            rx_ring,
        })
    }

    pub fn open(&mut self) -> nb::Result<(), IgbError> {
        match self.machine {
            Machine::Init => {
                self.reg.disable_interrupts();

                self.reg.modify_reg(|ctrl: CTRL| ctrl | CTRL::RST);

                self.machine = Machine::WaitForRst;

                Err(nb::Error::WouldBlock)
            }

            Machine::WaitForRst => {
                self.reg.wait_for(|reg: CTRL| !reg.contains(CTRL::RST))?;

                self.after_reset()
            }
            _ => Ok(()),
        }
    }

    fn after_reset(&mut self) -> nb::Result<(), IgbError> {
        self.reg.disable_interrupts();

        self.setup_phy_and_the_link()?;

        self.init_stat();

        self.init_rx();
        self.init_tx();

        //self.enable_interrupts();

        self.machine = Machine::Opened;
        Ok(())
    }

    fn init_stat(&mut self) {
        //TODO
    }

    fn init_rx(&mut self) {
        // disable rx when configing.
        self.reg.modify_reg::<RCTL>(|rctl| rctl ^ RCTL::RXEN);

        self.rx_ring.init();
    }

    fn init_tx(&mut self) {
        self.tx_ring.init();
    }

    fn setup_phy_and_the_link(&mut self) -> Result<(), IgbError> {
        Ok(())
    }

    pub fn mac(&self) -> [u8; 6] {
        self.reg.read_mac()
    }
}

#[derive(Debug, Clone, Copy)]
enum Machine {
    Init,
    WaitForRst,
    Opened,
}
