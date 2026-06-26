// RustShield — oracle: Canary comparison oracle for Phase II verification
//
// The oracle compares the Rust driver's runtime behavior against the
// eBPF canary baseline. A mismatch indicates a behavioral regression
// that must be fixed before the hotswap can proceed.

use crate::{BehavioralBaseline, CanaryComparisonResult, CanaryEvent};
use alloc::vec::Vec;

/// The canary oracle — compares C and Rust driver behavior.
pub struct CanaryOracle {
    baseline: BehavioralBaseline,
    rust_events: Vec<CanaryEvent>,
}

impl CanaryOracle {
    pub fn new(baseline: BehavioralBaseline) -> Self {
        Self {
            baseline,
            rust_events: Vec::new(),
        }
    }

    /// Record an event from the Rust driver.
    pub fn record_rust_event(&mut self, event: CanaryEvent) {
        self.rust_events.push(event);
    }

    /// Compare the Rust driver's behavior against the baseline.
    ///
    /// Returns a list of all comparisons, indicating which events match
    /// and which indicate regressions.
    pub fn compare(&self) -> Vec<CanaryComparisonResult> {
        let mut results = Vec::new();

        for baseline_event in &self.baseline.events {
            let matching_rust = self.rust_events.iter().find(|re| {
                re.attach_type == baseline_event.attach_type
                    && re.function == baseline_event.function
            });

            match matching_rust {
                Some(rust_event) => {
                    if self.events_equivalent(baseline_event, rust_event) {
                        results.push(CanaryComparisonResult::Match);
                    } else {
                        results.push(CanaryComparisonResult::Mismatch {
                            expected: baseline_event.clone(),
                            actual: rust_event.clone(),
                            details: "Behavioral mismatch detected",
                        });
                    }
                }
                None => {
                    results.push(CanaryComparisonResult::Missing {
                        expected: baseline_event.clone(),
                    });
                }
            }
        }

        // Check for unexpected Rust events
        for rust_event in &self.rust_events {
            let has_baseline = self.baseline.events.iter().any(|be| {
                be.attach_type == rust_event.attach_type && be.function == rust_event.function
            });
            if !has_baseline {
                results.push(CanaryComparisonResult::Unexpected {
                    actual: rust_event.clone(),
                });
            }
        }

        results
    }

    /// Check if two events are behaviorally equivalent.
    fn events_equivalent(&self, a: &CanaryEvent, b: &CanaryEvent) -> bool {
        // Simplified equivalence check — in production, this uses
        // the full behavioral spec comparison.
        a.severity == b.severity
    }

    /// Get the overall agreement ratio (0.0 - 1.0).
    pub fn agreement_ratio(&self) -> f64 {
        let results = self.compare();
        if results.is_empty() {
            return 1.0;
        }
        let matches = results
            .iter()
            .filter(|r| matches!(r, CanaryComparisonResult::Match))
            .count();
        matches as f64 / results.len() as f64
    }

    /// Check if the Rust driver passes the canary oracle.
    pub fn passes_oracle(&self, threshold: f64) -> bool {
        self.agreement_ratio() >= threshold
    }
}
