// RustShield — sysfs: Userspace interface for hotswap monitoring

use crate::HotswapPhase;
use crate::state_handoff;
use core::sync::atomic::{AtomicBool, Ordering};

static SYSFS_CREATED: AtomicBool = AtomicBool::new(false);

/// Create the sysfs interface at /sys/kernel/rustshield/
pub fn create_interface() -> Result<(), crate::HotswapError> {
    // In production, creates:
    //   /sys/kernel/rustshield/
    //   ├── status             (ro) - current hotswap phase
    //   ├── driver             (rw) - target driver name
    //   ├── rust_driver        (rw) - Rust module path
    //   ├── canary_status      (ro) - eBPF canary agreement status
    //   ├── last_dsc           (ro) - last device state capsule hex dump
    //   ├── proof_status       (ro) - Verus proof verification status
    //   ├── hotswap_count      (ro) - total successful hotswaps
    //   ├── rollback_count     (ro) - total rollbacks
    //   └── force_hotswap      (rw) - force flag
    SYSFS_CREATED.store(true, Ordering::Release);
    Ok(())
}

/// Remove the sysfs interface.
pub fn remove_interface() {
    SYSFS_CREATED.store(false, Ordering::Release);
}

/// Read the current hotswap status.
pub fn read_status() -> alloc::string::String {
    let phase = state_handoff::current_phase();
    format!("{:?}", phase)
}

/// Read the canary agreement status.
pub fn read_canary_status() -> alloc::string::String {
    // In production, queries the eBPF canary oracle
    "agree".into()
}

/// Read the proof verification status.
pub fn read_proof_status() -> alloc::string::String {
    // In production, queries the Verus proof checker
    "verified".into()
}
