// RustShield — error_codes: Error type definitions for the proof library

/// Error codes returned by Verus proof verification.
///
/// Maps each invariant to a unique error code for diagnostic purposes.
pub mod proof_error_codes {
    pub const INTERRUPT_SAFETY: u32 = 0x0001;
    pub const DMA_BOUNDARY: u32 = 0x0002;
    pub const REFCOUNT: u32 = 0x0003;
    pub const LOCK_ORDERING: u32 = 0x0004;
    pub const STATE_MACHINE: u32 = 0x0005;
    pub const IO_REGION: u32 = 0x0006;
    pub const TIMER_SAFETY: u32 = 0x0007;
    pub const WORKQUEUE: u32 = 0x0008;
    pub const DMA_COMPLETENESS: u32 = 0x0009;
    pub const IRQ_LIVENESS: u32 = 0x000A;
    pub const REGISTER_ACCESS: u32 = 0x000B;
    pub const POWER_STATE: u32 = 0x000C;
    pub const ERROR_RECOVERY: u32 = 0x000D;
    pub const MEMORY_LEAK: u32 = 0x000E;
}
