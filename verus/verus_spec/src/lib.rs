// RustShield — verus_spec: Shared specification language for driver behavior
//
// This crate defines the common specification types used by Verus proofs
// and the eBPF canary baseline. It provides the shared vocabulary for
// describing driver behavior across the RustShield components.

#![no_std]

pub mod driver_spec;
pub mod state_machine;
