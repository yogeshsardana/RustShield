// RustShield — canary: eBPF program loader and canary lifecycle manager

use crate::{CanaryAttachType, CanaryEvent};
use alloc::vec::Vec;

/// A deployed canary probe on a specific driver function.
#[derive(Debug, Clone)]
pub struct CanaryProbe {
    pub id: u32,
    pub driver_name: &'static str,
    pub attach_type: CanaryAttachType,
    pub function_name: &'static str,
    pub active: bool,
    pub events_collected: u64,
}

/// Manager for the eBPF canary lifecycle.
pub struct CanaryManager {
    probes: Vec<CanaryProbe>,
    event_buffer: Vec<CanaryEvent>,
    next_probe_id: u32,
}

impl CanaryManager {
    pub fn new() -> Self {
        Self {
            probes: Vec::new(),
            event_buffer: Vec::new(),
            next_probe_id: 1,
        }
    }

    /// Deploy a canary probe on a C driver function.
    ///
    /// In production, this:
    ///   1. Loads the eBPF program (BPF_PROG_TYPE_DRIVER_CANARY)
    ///   2. Attaches it to the target kprobe/tracepoint
    ///   3. Maps the canary ring buffer for event collection
    pub fn deploy_probe(
        &mut self,
        driver_name: &'static str,
        attach_type: CanaryAttachType,
        function_name: &'static str,
    ) -> u32 {
        let id = self.next_probe_id;
        self.next_probe_id += 1;
        self.probes.push(CanaryProbe {
            id,
            driver_name,
            attach_type,
            function_name,
            active: true,
            events_collected: 0,
        });
        id
    }

    /// Remove a deployed canary probe.
    pub fn remove_probe(&mut self, id: u32) {
        if let Some(probe) = self.probes.iter_mut().find(|p| p.id == id) {
            probe.active = false;
        }
    }

    /// Collect all pending events from the canary ring buffer.
    pub fn collect_events(&mut self) -> &[CanaryEvent] {
        // In production, reads from the BPF ring buffer map
        &self.event_buffer
    }

    /// Clear the event buffer.
    pub fn clear_events(&mut self) {
        self.event_buffer.clear();
    }

    /// Get the total number of events collected across all probes.
    pub fn total_events(&self) -> u64 {
        self.probes.iter().map(|p| p.events_collected).sum()
    }

    /// Check if all probes are still active.
    pub fn all_probes_active(&self) -> bool {
        self.probes.iter().all(|p| p.active)
    }
}
