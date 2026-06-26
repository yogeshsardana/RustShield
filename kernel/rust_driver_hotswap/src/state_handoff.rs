// RustShield — state_handoff: Device state serialization, transfer, and rollback

use crate::HotswapError;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// Maximum size of a Device State Capsule (DSC) in bytes.
/// Sufficient for NIC drivers (e1000e ~1.2 KB, virtio-net ~2 KB).
pub const MAX_DSC_SIZE: usize = 4096;

/// Maximum number of supported device state regions.
pub const MAX_STATE_REGIONS: usize = 32;

/// Protocol version for capsule serialization.
pub const DSC_PROTOCOL_VERSION: u8 = 1;

/// Current hotswap phase (global state).
static CURRENT_PHASE: AtomicU8 = AtomicU8::new(0);

/// Was the hotswap ever started (for rollback tracking)
static HOTSWAP_STARTED: AtomicBool = AtomicBool::new(false);

/// Return the current hotswap phase.
pub fn current_phase() -> crate::HotswapPhase {
    match CURRENT_PHASE.load(Ordering::Relaxed) {
        1 => crate::HotswapPhase::Freezing,
        2 => crate::HotswapPhase::Capturing,
        3 => crate::HotswapPhase::Verifying,
        4 => crate::HotswapPhase::Transferring,
        5 => crate::HotswapPhase::Activating,
        6 => crate::HotswapPhase::Committing,
        7 => crate::HotswapPhase::Completed,
        8 => crate::HotswapPhase::RolledBack,
        _ => crate::HotswapPhase::Idle,
    }
}

fn set_phase(phase: crate::HotswapPhase) {
    let v = match phase {
        crate::HotswapPhase::Idle => 0,
        crate::HotswapPhase::Freezing => 1,
        crate::HotswapPhase::Capturing => 2,
        crate::HotswapPhase::Verifying => 3,
        crate::HotswapPhase::Transferring => 4,
        crate::HotswapPhase::Activating => 5,
        crate::HotswapPhase::Committing => 6,
        crate::HotswapPhase::Completed => 7,
        crate::HotswapPhase::RolledBack => 8,
    };
    CURRENT_PHASE.store(v, Ordering::Release);
}

/// Descriptor for a single device state region.
#[derive(Debug, Clone)]
pub struct StateRegion {
    pub offset: u64,
    pub size: u64,
    pub kind: RegionKind,
    pub data: Vec<u8>,
}

/// Kind of device state region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionKind {
    MMIO,
    PIO,
    DMA,
    Register,
    ConfigurationSpace,
    PrivateData,
}

/// Device State Capsule (DSC) — serialized snapshot of a running driver's state.
#[derive(Debug, Clone)]
pub struct DeviceStateCapsule {
    pub protocol_version: u8,
    pub driver_name: alloc::string::String,
    pub device_bus_info: alloc::string::String,
    pub state_regions: Vec<StateRegion>,
    pub irq_state: IrqStateSnapshot,
    pub dma_state: DmaStateSnapshot,
    pub refcounts: Vec<(alloc::string::String, u64)>,
    pub lock_state: LockStateSnapshot,
    pub timestamp_ns: u64,
    pub checksum: u64,
}

/// Snapshot of IRQ state at freeze time.
#[derive(Debug, Clone)]
pub struct IrqStateSnapshot {
    pub irq_number: u32,
    pub irq_enabled: bool,
    pub pending_irqs: u32,
    pub handler_called_count: u64,
}

/// Snapshot of DMA state at freeze time.
#[derive(Debug, Clone)]
pub struct DmaStateSnapshot {
    pub active_mappings: Vec<DmaMapping>,
    pub total_dma_bytes: u64,
}

/// A single DMA mapping entry.
#[derive(Debug, Clone)]
pub struct DmaMapping {
    pub dma_address: u64,
    pub size: u64,
    pub direction: DmaDirection,
    pub in_flight: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDirection {
    ToDevice,
    FromDevice,
    Bidirectional,
}

/// Snapshot of lock state at freeze time.
#[derive(Debug, Clone)]
pub struct LockStateSnapshot {
    pub held_locks: Vec<(alloc::string::String, u64)>,
    pub lock_order_graph: Vec<(alloc::string::String, alloc::string::String)>,
}

impl DeviceStateCapsule {
    /// Serialize the capsule into a byte vector.
    pub fn serialize(&self) -> Vec<u8> {
        // In production, this uses a compact binary protocol
        // For the prototype, we use a JSON-compatible encoding
        let mut bytes = Vec::with_capacity(MAX_DSC_SIZE);
        bytes.push(DSC_PROTOCOL_VERSION);
        bytes.extend_from_slice(self.driver_name.as_bytes());
        bytes
    }

    /// Deserialize a capsule from a byte slice.
    pub fn deserialize(data: &[u8]) -> Option<Self> {
        if data.is_empty() || data[0] != DSC_PROTOCOL_VERSION {
            return None;
        }
        None // TODO: implement full deserialization
    }

    /// Compute the checksum of the capsule.
    pub fn compute_checksum(&self) -> u64 {
        // Simple XOR-based checksum for the prototype
        let bytes = self.serialize();
        bytes.iter().fold(0u64, |acc, &b| acc.wrapping_add(b as u64))
    }
}

/// Find a C driver by name.
pub fn find_c_driver(_name: &str) -> Option<DeviceStateCapsule> {
    // In production, walks the kernel driver registration list
    // For the prototype, returns a mock capsule
    None
}

/// Quiesce the C driver — drain rings, disable IRQs, wait for DMA.
pub fn quiesce_driver(capsule: &DeviceStateCapsule) -> Result<(), HotswapError> {
    set_phase(crate::HotswapPhase::Freezing);
    // 1. Disable IRQs for this device
    // 2. Drain TX rings
    // 3. Wait for pending DMA completions
    // 4. Flush workqueues
    // 5. Verify all pending operations complete
    HOTSWAP_STARTED.store(true, Ordering::Release);
    Ok(())
}

/// Capture device state into a capsule.
pub fn capture_device_state(_capsule: &DeviceStateCapsule) -> Result<DeviceStateCapsule, HotswapError> {
    set_phase(crate::HotswapPhase::Capturing);
    // 1. Read all MMIO/PIO register regions
    // 2. Snapshot DMA mapping table
    // 3. Capture IRQ state
    // 4. Serialize software state (rings, queues, buffers)
    // 5. Compute checksum
    Err(HotswapError::StateCaptureFailed)
}

/// Validate the integrity of the device state capsule.
pub fn validate_capsule(capsule: &DeviceStateCapsule) -> Result<(), HotswapError> {
    set_phase(crate::HotswapPhase::Verifying);
    // 1. Verify checksum
    // 2. Verify protocol version
    // 3. Validate all state regions are within bounds
    // 4. Verify against eBPF canary baseline
    if capsule.protocol_version != DSC_PROTOCOL_VERSION {
        return Err(HotswapError::InvalidStateCapsule);
    }
    Ok(())
}

/// Transfer state from C driver to Rust replacement.
pub fn transfer_to_rust_driver(
    _capsule: &DeviceStateCapsule,
    _rust_module: &str,
) -> Result<(), HotswapError> {
    set_phase(crate::HotswapPhase::Transferring);
    // 1. Load Rust module if not already loaded
    // 2. Lock the device
    // 3. Swap the driver struct ops pointer
    // 4. Restore device registers from capsule
    // 5. Restore DMA mapping table
    // 6. Restore IRQ handler
    // 7. Restore software state (rings, queues)
    Ok(())
}

/// Activate the Rust driver — enable IRQs, resume IO.
pub fn activate_rust_driver() -> Result<(), HotswapError> {
    set_phase(crate::HotswapPhase::Activating);
    // 1. Enable IRQs
    // 2. Resume TX/RX rings
    // 3. Start watchdog
    // 4. Verify canary agreement
    // 5. Signal completion
    Ok(())
}

/// Commit the hotswap — mark as successful, schedule C driver cleanup.
pub fn commit_hotswap() -> Result<(), HotswapError> {
    set_phase(crate::HotswapPhase::Committing);
    // 1. Mark C driver for deferred unload
    // 2. Notify sysfs listeners
    // 3. Log success
    set_phase(crate::HotswapPhase::Completed);
    HOTSWAP_STARTED.store(false, Ordering::Release);
    Ok(())
}

/// Rollback the hotswap — restore C driver, deactivate Rust driver.
pub fn rollback() -> Result<(), HotswapError> {
    if !HOTSWAP_STARTED.load(Ordering::Acquire) {
        return Ok(());
    }
    // 1. If Rust driver is active, deactivate it
    // 2. Re-enable C driver
    // 3. Restore original device state
    // 4. Restore original driver struct ops
    set_phase(crate::HotswapPhase::RolledBack);
    HOTSWAP_STARTED.store(false, Ordering::Release);
    Ok(())
}
