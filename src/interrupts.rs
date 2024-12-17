use alloc::vec::Vec;

/// The number of msi-x vectors this device can have.
/// It can be set from PCI space, but we took the value from the data sheet.
pub const IXGBE_MAX_MSIX_VECTORS: usize = 64;

#[derive(Default)]
pub struct Interrupts {
    pub interrupts_enabled: bool,     // Interrupts for this device enabled?
    pub itr_rate: u32,                // Interrupt Throttling Rate
    pub interrupt_type: u64,          // MSI or MSIX
    pub timeout_ms: i16,              // Interrupt timeout in ms (-1 to disable timeout)
    pub queues: Vec<InterruptsQueue>, // Interrupt settings per queue
}

pub struct InterruptsQueue {
    pub interrupt_enabled: bool, // Interrupt for this queue enabled?
}

pub enum InterruptType {
    Msi,
    Msix,
}
