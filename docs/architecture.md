# RustShield Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            USERSPACE                                         │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  rustshield-migrate CLI                                              │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐    │  │
│  │  │ analyze  │→│ skeleton │→│ verify  │→│ report           │    │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘    │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                           │ ioctl (DRIVER_HOTSWAP_COMMIT)                   │
└───────────────────────────┼─────────────────────────────────────────────────┘
                            │
┌───────────────────────────┼─────────────────────────────────────────────────┐
│                     KERNEL SPACE                                            │
│  ┌────────────────────────▼──────────────────────────────────────────┐     │
│  │  rust_driver_hotswap (Kernel Subsystem)                           │     │
│  │  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌────────────────┐  │     │
│  │  │  ioctl   │→│  state_   │→│ protocol │→│ sysfs          │  │     │
│  │  │  handler │  │  handoff  │  │  state   │  │ (/sys/kernel/  │  │     │
│  │  │          │  │           │  │  machine  │  │  rustshield/)  │  │     │
│  │  └──────────┘  └───────────┘  └──────────┘  └────────────────┘  │     │
│  └──────────────────────────────────────────────────────────────────┘     │
│                           │                                                 │
│  ┌────────────────────────▼──────────────────────────┐                     │
│  │  bpf_driver_canary                                │                     │
│  │  ┌──────────┐  ┌───────────┐  ┌──────────┐      │                     │
│  │  │ canary   │→│ baseline  │→│ oracle   │      │                     │
│  │  │ probes   │  │ generator │  │ comparator│      │                     │
│  │  └──────────┘  └───────────┘  └──────────┘      │                     │
│  └──────────────────────────────────────────────────────┘                 │
│                           │                                                 │
│  ┌────────────────────────▼──────────────────────────┐                     │
│  │  C Driver (existing)           Rust Driver (new)  │                     │
│  │  ┌────────────────────┐      ┌──────────────────┐ │                     │
│  │  │ e1000e (C)        │      │ rust-e1000e      │ │                     │
│  │  │ - probe/remove    │      │ - probe/remove   │ │                     │
│  │  │ - open/close      │◄────►│ - open/close     │ │                     │
│  │  │ - start_xmit      │SWAP  │ - start_xmit     │ │                     │
│  │  │ - IRQ handler     │      │ - IRQ handler    │ │                     │
│  │  └────────────────────┘      └──────────────────┘ │                     │
│  └──────────────────────────────────────────────────────┘                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Architecture

### 1. `rust_driver_hotswap` — Kernel Subsystem

The central coordinator that provides:

- **Character device** (`/dev/rustshield`) for ioctl-based hotswap control
- **`DRIVER_HOTSWAP_COMMIT` ioctl** — the core operation that triggers the 6-phase protocol
- **Device State Capsule (DSC)** — serialization format for capturing driver state
- **State handoff engine** — manages FREEZE → CAPTURE → VERIFY → TRANSFER → ACTIVATE → COMMIT
- **Rollback mechanism** — can recover from any phase before COMMIT if verification fails
- **Sysfs interface** — `/sys/kernel/rustshield/` for monitoring and control

### 2. `verus_kernel_proofs` — Formal Proof Library

Provides 14 proof modules that correspond to known Linux kernel driver CVE classes:

- Each proof module defines a `*Witness` type that tracks proof state
- Proofs are checked via Verus at compile time, not at runtime
- Driver authors implement `DriverSafetyContract` trait to opt in
- Proofs compose: verifying against all 14 invariants is a single function call

### 3. `bpf_driver_canary` — eBPF Behavioral Oracle

A new eBPF program type (`BPF_PROG_TYPE_DRIVER_CANARY`) that:

- Attaches to C driver functions via kprobes and tracepoints
- Captures behavioral traces without modifying C source
- Produces a serialized `BehavioralBaseline` specification
- Compares Rust driver behavior at runtime against the baseline
- Provides an agreement ratio ([0.0, 1.0]) for migration readiness

### 4. `rustshield-migrate` — CLI Tool

Three commands:

- `analyze` — Scans C source, identifies IO operations, state variables, locks, DMA regions
- `skeleton` — Generates Rust driver code with optional Verus annotations
- `verify` — Runs Verus proofs and eBPF canary comparison

## Hotswap Protocol Detail

```
Phase 1: FREEZE (C Driver Quiesce)
├── Disable interrupts (IRQ masking)
├── Drain TX/RX rings to completion
├── Wait for pending DMA operations
├── Flush workqueues and tasklets
└── Verify all in-flight operations complete

Phase 2: CAPTURE (State Serialization)
├── Read all MMIO/PIO register regions
├── Snapshot DMA mapping table
├── Capture interrupt state (pending, enabled)
├── Serialize software state (rings, queues)
└── Compute device state capsule checksum

Phase 3: VERIFY (Capsule Validation)
├── Verify capsule checksum
├── Validate all state region bounds
├── Compare state against eBPF canary baseline
└── Verify Rust driver proofs are current

Phase 4: TRANSFER (State Migration)
├── Load Rust module (if not loaded)
├── Acquire device lock
├── Swap driver struct ops pointer
├── Restore device registers from capsule
├── Restore DMA mapping table
├── Restore IRQ handler registration
└── Restore software state (rings, queues)

Phase 5: ACTIVATE (Rust Driver Enable)
├── Enable interrupts on Rust driver
├── Resume TX/RX rings
├── Start watchdog timer
├── Verify canary agreement (post-activation)
└── Signal completion to userspace via ioctl return

Phase 6: COMMIT (Finalization)
├── Mark C driver for deferred unload
├── Notify sysfs listeners of success
├── Log hotswap event
└── Schedule C driver module cleanup
```

## Safety Guarantees

1. **Atomicity**: The handoff window is sub-millisecond. During this window, interrupts are masked and the device lock is held — no external observer sees an inconsistent state.

2. **Verification**: The Rust driver is formally verified before activation. The eBPF canary oracle compares behavioral traces — any mismatch aborts the migration.

3. **Rollback**: From any phase before COMMIT, the system can roll back to the C driver. The C driver is never unloaded until the Rust driver confirms it is handling traffic correctly.

4. **No Data Loss**: Hardware buffers (NIC FIFOs, DMA rings) remain operational during the switch. The kernel's networking stack is not modified — only the driver struct ops pointer changes.
