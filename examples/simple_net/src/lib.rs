// RustShield — simple_net: Minimal NIC driver example for proof-of-concept

#![no_std]

use kernel::prelude::*;

/// Minimal device data for the simple NIC.
pub struct SimpleNetDevice {
    pub io_base: kernel::io::IoMem,
    pub irq: u32,
    pub mac_address: [u8; 6],
    pub link_up: bool,
}

impl SimpleNetDevice {
    pub fn new(io_base: kernel::io::IoMem, irq: u32) -> Result<Self> {
        Ok(Self {
            io_base,
            irq,
            mac_address: [0x02, 0x00, 0x00, 0x00, 0x00, 0x01],
            link_up: false,
        })
    }

    pub fn reset(&mut self) {
        self.link_up = false;
    }
}

/// Probe function.
pub fn probe(pdev: &mut kernel::platform::Device) -> Result<SimpleNetDevice> {
    let io_base = kernel::io::IoMem::try_new(pdev)?;
    let irq = pdev.irq_number()?;
    let dev = SimpleNetDevice::new(io_base, irq)?;
    pr_info!("simple-net: Device probed\n");
    Ok(dev)
}
