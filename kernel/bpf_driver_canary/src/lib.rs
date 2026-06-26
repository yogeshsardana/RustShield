// RustShield — bpf_driver_canary: eBPF-based driver behavioral oracle
//
// This module defines BPF_PROG_TYPE_DRIVER_CANARY, a new eBPF program type
// for shadowing C driver critical paths and producing behavioral baselines.
//
// The canary system:
//   1. Attaches kprobes/tracepoints to C driver entry/exit points
//   2. Captures IO path behavior (register reads/writes, DMA operations)
//   3. Records state transitions and memory access patterns
//   4. Produces a serialized behavioral specification
//   5. Compares Rust driver behavior against the oracle

#![no_std]
#![feature(allocator_api)]

extern crate alloc;

mod canary;
mod baseline;
mod oracle;

pub use canary::*;
pub use baseline::*;
pub use oracle::*;

/// The eBPF program type code for driver canaries.
/// This would be registered in the kernel's bpf_types.h
pub const BPF_PROG_TYPE_DRIVER_CANARY: u32 = 40;

/// Canary program attach types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanaryAttachType {
    /// Attach to driver .probe function entry
    ProbeEntry,
    /// Attach to driver .probe function exit
    ProbeExit,
    /// Attach to driver .open function entry
    OpenEntry,
    /// Attach to driver .open function exit
    OpenExit,
    /// Attach to driver .ndo_start_xmit entry
    XmitEntry,
    /// Attach to driver .ndo_start_xmit exit
    XmitExit,
    /// Attach to interrupt handler entry
    IrqHandlerEntry,
    /// Attach to interrupt handler exit
    IrqHandlerExit,
    /// Attach to DMA alloc function
    DmaAlloc,
    /// Attach to DMA free function
    DmaFree,
    /// Attach to register read
    RegisterRead,
    /// Attach to register write
    RegisterWrite,
    /// Attach to device state transition
    StateTransition,
    /// Custom tracepoint
    Custom(&'static str),
}

/// Canary event severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanarySeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

/// A single canary observation event.
#[derive(Debug, Clone)]
pub struct CanaryEvent {
    pub timestamp_ns: u64,
    pub attach_type: CanaryAttachType,
    pub function: &'static str,
    pub cpu_id: u32,
    pub pid: u32,
    pub severity: CanarySeverity,
    pub data: CanaryEventData,
}

/// Data payload of a canary event.
#[derive(Debug, Clone)]
pub enum CanaryEventData {
    RegisterRead {
        reg_offset: u64,
        value: u64,
        width: u8,
    },
    RegisterWrite {
        reg_offset: u64,
        value: u64,
        width: u8,
    },
    DmaOperation {
        direction: DmaDirection,
        addr: u64,
        size: u64,
    },
    StateChange {
        from: u8,
        to: u8,
    },
    IrqEvent {
        irq_num: u32,
        action: IrqAction,
    },
    IoOperation {
        kind: IoOpKind,
        offset: u64,
        len: u64,
    },
    GenericData {
        key: u64,
        value: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDirection {
    ToDevice,
    FromDevice,
    Bidirectional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrqAction {
    Raise,
    Acknowledge,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOpKind {
    Read,
    Write,
    Poll,
}

/// Result of comparing a Rust driver's behavior against the canary baseline.
#[derive(Debug, Clone)]
pub enum CanaryComparisonResult {
    Match,
    Mismatch {
        expected: CanaryEvent,
        actual: CanaryEvent,
        details: &'static str,
    },
    Missing {
        expected: CanaryEvent,
    },
    Unexpected {
        actual: CanaryEvent,
    },
}

/// Check that the canary subsystem is supported on the current kernel.
pub fn is_canary_supported() -> bool {
    // In production, checks kernel config for BPF_SYSCALL
    true
}
