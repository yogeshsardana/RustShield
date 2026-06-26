// RustShield — driver_spec: Driver specification types shared across components

use core::fmt;

/// Unique identifier for a driver within the RustShield system.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DriverId {
    pub name: &'static str,
    pub bus: BusType,
    pub vendor_id: u16,
    pub device_id: u16,
}

/// Bus type supported by the driver.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BusType {
    Pci,
    Platform,
    Usb,
    Virtio,
    I2c,
    Spi,
}

/// Driver operation kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverOp {
    Probe,
    Remove,
    Open,
    Release,
    StartXmit,
    Ioctl,
    IrqHandler,
    DmaAlloc,
    DmaFree,
    RegisterRead,
    RegisterWrite,
    StateTransition,
}

/// Result of a driver operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpResult {
    Success,
    Error(u32),
    Retry,
}

/// A driver capability level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapabilityLevel {
    /// Driver has been analyzed but not yet annotated
    Analyzed,
    /// Driver has Rust skeleton with Verus annotations
    Annotated,
    /// Driver has been formally verified against invariants
    Verified,
    /// Driver has passed eBPF canary comparison
    CanaryPassed,
    /// Driver has been successfully hotswapped
    Migrated,
}
