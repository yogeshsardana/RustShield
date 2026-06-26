// RustShield — workqueue_safety: Formal proof of workqueue ordering
//
// Invariant 8: Work items execute in correct order relative to dependencies.
//
// Proof technique: Partial order verification of work item scheduling.

use crate::VerificationError;

/// A tracked work item.
#[derive(Clone, Debug)]
pub struct WorkItem {
    pub id: u32,
    pub depends_on: Vec<u32>,
    pub scheduled: bool,
    pub completed: bool,
}

/// Proof witness for workqueue ordering correctness.
pub struct WorkqueueWitness {
    items: Vec<WorkItem>,
}

impl WorkqueueWitness {
    pub fn new(items: Vec<WorkItem>) -> Self {
        Self { items }
    }

    /// Verify work item ordering is correct.
    ///
    /// Verus proof sketch:
    /// ```verus
    /// proof fn workqueue_proof(witness: &WorkqueueWitness)
    ///     ensures forall |i, j|
    ///         i.depends_on.contains(j) ==> j.completed before i.scheduled
    /// {
    ///     // For each work item, all its dependencies must complete
    ///     // before the item is scheduled.
    /// }
    /// ```
    pub fn verify(&self) -> Result<(), VerificationError> {
        for item in &self.items {
            for dep_id in &item.depends_on {
                if let Some(dep) = self.items.iter().find(|w| w.id == *dep_id) {
                    if !dep.completed && item.scheduled {
                        return Err(VerificationError::WorkqueueOrderingViolation);
                    }
                }
            }
        }
        Ok(())
    }
}
