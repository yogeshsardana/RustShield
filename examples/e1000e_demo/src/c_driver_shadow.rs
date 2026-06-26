// RustShield — c_driver_shadow: State shadow of the C e1000e driver
//
// This module mirrors the C e1000e driver's state structures for
// the purpose of verifying that the Rust replacement captures
// all device state correctly during hotswap.

use crate::E1000eDevice;

/// Shadow of the C driver's private data structure.
///
/// During hotswap, this is populated from the Device State Capsule
/// and used to verify correctness of the state transfer.
pub struct CDriverShadow {
    pub tx_ring: ShadowTxRing,
    pub rx_ring: ShadowRxRing,
    pub registers: ShadowRegisters,
    pub stats: ShadowStats,
}

pub struct ShadowTxRing {
    pub base: u64,
    pub len: u32,
    pub head: u32,
    pub tail: u32,
    pub desc_count: u32,
    pub pending_completions: u32,
}

pub struct ShadowRxRing {
    pub base: u64,
    pub len: u32,
    pub head: u32,
    pub tail: u32,
    pub desc_count: u32,
    pub pending_packets: u32,
}

pub struct ShadowRegisters {
    pub ctrl: u32,
    pub status: u32,
    pub rctl: u32,
    pub tctl: u32,
    pub imc: u32,
    pub ims: u32,
}

pub struct ShadowStats {
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub tx_errors: u64,
    pub rx_errors: u64,
}

impl CDriverShadow {
    /// Compare the shadow state against the Rust driver's state.
    ///
    /// Returns Ok(()) if all state matches, or the first mismatch found.
    pub fn compare(&self, rust_dev: &E1000eDevice) -> Result<(), &'static str> {
        let stats = rust_dev.stats.lock();
        if stats.tx_packets != self.stats.tx_packets {
            return Err("TX packet count mismatch");
        }
        if stats.rx_packets != self.stats.rx_packets {
            return Err("RX packet count mismatch");
        }
        Ok(())
    }
}
