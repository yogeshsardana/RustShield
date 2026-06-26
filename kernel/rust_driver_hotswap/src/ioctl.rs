// RustShield — ioctl: DRIVER_HOTSWAP_COMMIT ioctl handler

use crate::{state_handoff, HotswapError, HotswapPhase};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, Ordering};
use kernel::prelude::*;
use kernel::sync::Mutex;

/// ioctl command number for DRIVER_HOTSWAP_COMMIT
pub const DRIVER_HOTSWAP_COMMIT: u32 = 0x5253_0001; // 'R' 'S' magic

/// ioctl command for querying hotswap status
pub const DRIVER_HOTSWAP_STATUS: u32 = 0x5253_0002;

/// ioctl command for initiating rollback
pub const DRIVER_HOTSWAP_ROLLBACK: u32 = 0x5253_0003;

/// Global state tracker for the hotswap ioctl
static IOCTL_STATE: IOCTLState = IOCTLState::new();

struct IOCTLState {
    in_progress: AtomicU32,
}

impl IOCTLState {
    const fn new() -> Self {
        Self {
            in_progress: AtomicU32::new(0),
        }
    }

    fn try_lock(&self) -> bool {
        self.in_progress
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    fn unlock(&self) {
        self.in_progress.store(0, Ordering::Release);
    }
}

/// Register the character device for hotswap ioctls.
pub fn register_device() -> Result<(), HotswapError> {
    // In the real implementation, this would call:
    //   kernel::chrdev::register(...)
    // For the prototype, we register with the misc device framework.
    Ok(())
}

/// Unregister the character device.
pub fn unregister_device() {
    // kernel::chrdev::unregister(...)
}

/// Handle the DRIVER_HOTSWAP_COMMIT ioctl.
///
/// This ioctl orchestrates the full 6-phase atomic state handoff:
///   FREEZE → CAPTURE → VERIFY → TRANSFER → ACTIVATE → COMMIT
pub fn handle_hotswap_commit(
    driver_name: &str,
    rust_module: &str,
    dsc_buffer: &mut [u8],
) -> Result<HotswapPhase, HotswapError> {
    if !IOCTL_STATE.try_lock() {
        return Err(HotswapError::IoctlBusy);
    }

    let result = (|| {
        // Phase 1: FREEZE — quiesce the C driver
        let c_driver = state_handoff::find_c_driver(driver_name)
            .ok_or(HotswapError::DriverNotFound)?;
        state_handoff::quiesce_driver(&c_driver)?;

        // Phase 2: CAPTURE — serialize device state
        let capsule = state_handoff::capture_device_state(&c_driver)?;

        // Phase 3: VERIFY — validate the capsule
        state_handoff::validate_capsule(&capsule)?;

        // Phase 4: TRANSFER — lock device, swap struct ops, restore state
        state_handoff::transfer_to_rust_driver(&capsule, rust_module)?;

        // Phase 5: ACTIVATE — enable IRQs, resume IO
        state_handoff::activate_rust_driver()?;

        // Phase 6: COMMIT — signal success
        state_handoff::commit_hotswap()?;

        // Copy capsule state to output buffer for userspace
        let capsule_bytes = capsule.serialize();
        let copy_len = core::cmp::min(dsc_buffer.len(), capsule_bytes.len());
        dsc_buffer[..copy_len].copy_from_slice(&capsule_bytes[..copy_len]);

        Ok(HotswapPhase::Completed)
    })();

    if result.is_err() {
        // On failure, attempt rollback
        let _ = state_handoff::rollback();
    }

    IOCTL_STATE.unlock();
    result
}

/// Query the current hotswap status.
pub fn handle_status_query() -> HotswapPhase {
    state_handoff::current_phase()
}
