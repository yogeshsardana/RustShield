// RustShield — protocol: Three-phase hotswap protocol state machine
//
// Defines the protocol state machine that governs transitions
// between hotswap phases, ensuring safety invariants are maintained.

use crate::HotswapPhase;
use crate::HotswapError;

/// Maximum time (in microseconds) to wait for driver quiesce.
pub const FREEZE_TIMEOUT_US: u64 = 500;

/// Maximum time (in microseconds) for the full hotswap window.
pub const HOTSWAP_WINDOW_US: u64 = 1000;

/// Maximum number of retries for state verification.
pub const VERIFY_RETRIES: u32 = 3;

/// Protocol state machine for the hotswap sequence.
///
/// Ensures:
/// - Phases are transitioned in the correct order
/// - Each phase completes before the next begins
/// - Rollback is possible from any phase before COMMIT
/// - Timeouts are enforced
pub struct HotswapProtocol {
    current_phase: HotswapPhase,
    started_at: u64,
    freeze_deadline: u64,
    hotswap_deadline: u64,
}

impl HotswapProtocol {
    /// Create a new protocol instance for a hotswap operation.
    pub fn new() -> Self {
        let now = Self::timestamp_us();
        Self {
            current_phase: HotswapPhase::Idle,
            started_at: now,
            freeze_deadline: now + FREEZE_TIMEOUT_US,
            hotswap_deadline: now + HOTSWAP_WINDOW_US,
        }
    }

    /// Transition to the next phase.
    pub fn transition_to(&mut self, next: HotswapPhase) -> Result<(), HotswapError> {
        if !self.current_phase.can_transition_to(next) {
            return Err(HotswapError::InvalidStateCapsule);
        }

        if self.is_timed_out() {
            return Err(HotswapError::DeviceLockTimeout);
        }

        self.current_phase = next;
        Ok(())
    }

    /// Check whether the hotswap window has expired.
    pub fn is_timed_out(&self) -> bool {
        Self::timestamp_us() > self.hotswap_deadline
    }

    /// Get the remaining time in the hotswap window (microseconds).
    pub fn remaining_us(&self) -> i64 {
        self.hotswap_deadline as i64 - Self::timestamp_us() as i64
    }

    /// Check whether we are past the freeze deadline.
    pub fn freeze_overdue(&self) -> bool {
        Self::timestamp_us() > self.freeze_deadline
    }

    /// Get the current phase.
    pub fn phase(&self) -> HotswapPhase {
        self.current_phase
    }

    /// Check if rollback is still possible.
    pub fn can_rollback(&self) -> bool {
        matches!(
            self.current_phase,
            HotswapPhase::Freezing
                | HotswapPhase::Capturing
                | HotswapPhase::Verifying
                | HotswapPhase::Transferring
                | HotswapPhase::Activating
        )
    }

    /// Determine which phase to roll back to.
    pub fn rollback_target(&self) -> HotswapPhase {
        // Rollback to the idle state; the rollback handler
        // will re-enable the C driver.
        HotswapPhase::RolledBack
    }

    /// Get a monotonic timestamp in microseconds.
    fn timestamp_us() -> u64 {
        // In the kernel, this would use ktime_get()
        // For the prototype, we use a simple counter.
        0
    }
}

/// Validate that a given driver family is supported for hotswap.
pub fn validate_driver_family(driver_name: &str) -> Result<(), HotswapError> {
    // Supported driver families
    const SUPPORTED: &[&str] = &[
        "e1000e",
        "virtio_net",
        "virtio_blk",
        "nvme",
        "igb",
        "ixgbe",
        "mlx5_core",
        "i40e",
    ];

    if SUPPORTED.contains(&driver_name) {
        Ok(())
    } else {
        Err(HotswapError::IncompatibleDriverFamily)
    }
}

/// Bit flags for the DRIVER_HOTSWAP_COMMIT ioctl argument.
///
/// These control the behavior of the hotswap operation.
pub mod flags {
    /// Force hotswap even if canary verification fails.
    pub const FORCE: u32 = 1 << 0;
    /// Dry-run — perform all phases except the actual COMMIT.
    pub const DRY_RUN: u32 = 1 << 1;
    /// Skip Verus proof re-verification (use cached proofs).
    pub const SKIP_VERIFY: u32 = 1 << 2;
    /// Enable verbose logging during hotswap.
    pub const VERBOSE: u32 = 1 << 3;
    /// Nano profile for embedded drivers (single-phase).
    pub const NANO: u32 = 1 << 4;
}
