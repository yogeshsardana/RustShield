// RustShield — timer_safety: Formal proof of timer cancellation correctness
//
// Invariant 7: Timers are cancelled before module unload.
//
// Proof technique: Temporal logic — a timer that fires after
// module unload causes use-after-free.

use crate::VerificationError;

/// State of a kernel timer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerState {
    Stopped,
    Armed,
    Firing,
    Cancelled,
}

/// A proof-tracked timer.
pub struct TrackedTimer {
    id: u32,
    state: TimerState,
}

/// Proof witness for timer safety.
pub struct TimerSafetyWitness {
    timers: Vec<TrackedTimer>,
}

impl TimerSafetyWitness {
    pub fn new() -> Self {
        Self {
            timers: Vec::new(),
        }
    }

    pub fn add_timer(&mut self, id: u32) {
        self.timers.push(TrackedTimer {
            id,
            state: TimerState::Stopped,
        });
    }

    pub fn arm(&mut self, id: u32) {
        if let Some(t) = self.timers.iter_mut().find(|t| t.id == id) {
            t.state = TimerState::Armed;
        }
    }

    pub fn cancel(&mut self, id: u32) {
        if let Some(t) = self.timers.iter_mut().find(|t| t.id == id) {
            t.state = TimerState::Cancelled;
        }
    }

    /// Verify all timers are cancelled at module unload.
    ///
    /// Verus proof sketch:
    /// ```verus
    /// proof fn timer_safety_proof(witness: &TimerSafetyWitness)
    ///     ensures forall |t: TrackedTimer|
    ///         t.state == TimerState::Cancelled || t.state == TimerState::Stopped
    /// {
    ///     // At module exit, every timer must be stopped or cancelled.
    ///     // An armed timer that fires after deallocation is UB.
    /// }
    /// ```
    pub fn verify(&self) -> Result<(), VerificationError> {
        for timer in &self.timers {
            if timer.state == TimerState::Armed || timer.state == TimerState::Firing {
                return Err(VerificationError::TimerNotCancelled);
            }
        }
        Ok(())
    }
}
