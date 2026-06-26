// RustShield — rust_driver_hotswap: Kernel driver hotswap subsystem
//
// This module implements the DRIVER_HOTSWAP_COMMIT ioctl and the
// Device State Capsule (DSC) protocol for live migration of kernel drivers.

#![no_std]
#![feature(allocator_api)]
#![feature(associated_type_defaults)]

extern crate alloc;

mod ioctl;
mod state_handoff;
mod protocol;
mod sysfs;

use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, Ordering};

/// Global hotswap subsystem state
static HOTSWAP_ENABLED: AtomicBool = AtomicBool::new(false);

/// Initialize the rust_driver_hotswap subsystem.
///
/// Registers the character device for DRIVER_HOTSWAP_COMMIT ioctl,
/// initializes the sysfs interface, and prepares the state handoff
/// infrastructure.
pub fn init() -> Result<(), HotswapError> {
    if HOTSWAP_ENABLED.swap(true, Ordering::Acquire) {
        return Err(HotswapError::AlreadyInitialized);
    }

    ioctl::register_device().map_err(|e| {
        HOTSWAP_ENABLED.store(false, Ordering::Release);
        e
    })?;

    sysfs::create_interface().map_err(|e| {
        ioctl::unregister_device();
        HOTSWAP_ENABLED.store(false, Ordering::Release);
        e
    })?;

    pr_info!("RustShield: rust_driver_hotswap initialized\n");
    Ok(())
}

/// Shut down the hotswap subsystem.
pub fn shutdown() {
    if !HOTSWAP_ENABLED.swap(false, Ordering::Acquire) {
        return;
    }
    sysfs::remove_interface();
    ioctl::unregister_device();
    pr_info!("RustShield: rust_driver_hotswap shutdown\n");
}

/// Errors that can occur during hotswap operations.
#[derive(Debug, Clone, Copy)]
pub enum HotswapError {
    AlreadyInitialized,
    DeviceRegistrationFailed,
    SysfsCreationFailed,
    DriverNotFound,
    DriverNotQuiesced,
    StateCaptureFailed,
    StateVerificationFailed,
    StateTransferFailed,
    RustDriverLoadFailed,
    RustDriverActivationFailed,
    IncompatibleDriverFamily,
    DeviceLockTimeout,
    CanaryMismatch,
    InvalidStateCapsule,
    ProofVerificationFailed,
    IoctlInvalidArgument,
    IoctlPermissionDenied,
    IoctlBusy,
}

/// Hotswap phase indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotswapPhase {
    Idle,
    Freezing,
    Capturing,
    Verifying,
    Transferring,
    Activating,
    Committing,
    Completed,
    RolledBack,
}

impl HotswapPhase {
    pub fn can_transition_to(&self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Idle, Self::Freezing)
                | (Self::Freezing, Self::Capturing)
                | (Self::Capturing, Self::Verifying)
                | (Self::Verifying, Self::Transferring)
                | (Self::Transferring, Self::Activating)
                | (Self::Activating, Self::Committing)
                | (Self::Committing, Self::Completed)
                | (_, Self::RolledBack)
        )
    }
}
