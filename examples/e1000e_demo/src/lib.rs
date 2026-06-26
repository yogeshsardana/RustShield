// RustShield — e1000e Demo: Rust replacement driver for Intel e1000e
//
// This is a simplified Rust implementation of the e1000e NIC driver
// used in the live hotswap demonstration. It implements the subset
// of operations needed for network I/O and hotswap verification.

#![no_std]
#![feature(allocator_api)]

extern crate alloc;

use alloc::boxed::Box;
use kernel::prelude::*;
use kernel::sync::Mutex;

pub mod c_driver_shadow;
pub mod hotswap_demo;

/// e1000e device registers (subset for demo).
#[repr(C)]
pub struct E1000eRegs {
    pub ctrl: u32,       // Device Control Register
    pub status: u32,     // Device Status Register
    pub eecd: u32,       // EEPROM/Flash Control
    pub eerd: u32,       // EEPROM Read
    pub ctrlext: u32,    // Extended Device Control
    pub mdic: u32,       // MDI Control
    pub fcrtl: u32,      // Flow Control Receive Threshold Low
    pub fcrth: u32,      // Flow Control Receive Threshold High
    pub rctl: u32,       // Receive Control
    pub tctl: u32,       // Transmit Control
    pub rdbal: u32,      // Receive Descriptor Base Address Low
    pub rdbah: u32,      // Receive Descriptor Base Address High
    pub rdlen: u32,      // Receive Descriptor Length
    pub rdh: u32,        // Receive Descriptor Head
    pub rdt: u32,        // Receive Descriptor Tail
    pub tdbal: u32,      // Transmit Descriptor Base Address Low
    pub tdbah: u32,      // Transmit Descriptor Base Address High
    pub tdlen: u32,      // Transmit Descriptor Length
    pub tdh: u32,        // Transmit Descriptor Head
    pub tdt: u32,        // Transmit Descriptor Tail
    pub imc: u32,        // Interrupt Mask Clear
    pub ims: u32,        // Interrupt Mask Set
    pub icr: u32,        // Interrupt Cause Read
}

/// Driver private data for the Rust e1000e replacement.
pub struct E1000eDevice {
    pub regs: kernel::io::IoMem,
    pub irq: u32,
    pub state: Mutex<DeviceState>,
    pub tx_ring: Option<Box<TxRing>>,
    pub rx_ring: Option<Box<RxRing>>,
    pub stats: Mutex<DeviceStats>,
}

/// Device operational state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Down,
    Up,
    Suspended,
    Error,
}

/// TX descriptor ring.
pub struct TxRing {
    pub base: u64,
    pub len: u32,
    pub head: u32,
    pub tail: u32,
    pub desc: Vec<TxDesc>,
}

/// RX descriptor ring.
pub struct RxRing {
    pub base: u64,
    pub len: u32,
    pub head: u32,
    pub tail: u32,
    pub desc: Vec<RxDesc>,
}

/// e1000e TX descriptor.
#[repr(C)]
pub struct TxDesc {
    pub addr: u64,
    pub length: u16,
    pub cso: u8,
    pub cmd: u8,
    pub status: u8,
    pub css: u8,
    pub special: u16,
}

/// e1000e RX descriptor.
#[repr(C)]
pub struct RxDesc {
    pub addr: u64,
    pub length: u16,
    pub csum: u16,
    pub status: u8,
    pub errors: u8,
    pub special: u16,
}

/// Device statistics.
#[derive(Debug, Clone, Copy)]
pub struct DeviceStats {
    pub tx_packets: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub rx_bytes: u64,
    pub irq_count: u64,
    pub tx_errors: u64,
    pub rx_errors: u64,
    pub dropped: u64,
}

impl E1000eDevice {
    pub fn new(regs: kernel::io::IoMem, irq: u32) -> Result<Box<Self>> {
        let dev = Box::try_new(Self {
            regs,
            irq,
            state: Mutex::new(DeviceState::Down),
            tx_ring: None,
            rx_ring: None,
            stats: Mutex::new(DeviceStats {
                tx_packets: 0,
                tx_bytes: 0,
                rx_packets: 0,
                rx_bytes: 0,
                irq_count: 0,
                tx_errors: 0,
                rx_errors: 0,
                dropped: 0,
            }),
        })?;
        Ok(dev)
    }

    pub fn reset(&self) {
        // Reset device to known state
        let mut state = self.state.lock();
        *state = DeviceState::Down;
    }
}

/// Probe function — called by kernel when device is detected.
pub fn probe(pdev: &mut kernel::platform::Device) -> Result<Box<E1000eDevice>> {
    let regs = kernel::io::IoMem::try_new(pdev)?;
    let irq = pdev.irq_number()?;
    let dev = E1000eDevice::new(regs, irq)?;
    dev.reset();
    pr_info!("rust-e1000e: Device probed\n");
    Ok(dev)
}
