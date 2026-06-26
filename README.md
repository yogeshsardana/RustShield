# RustShield

> Memory-Safe Linux Kernel Driver Hotswap with Formal Verification via Verus + eBPF Canaries

[![CI](https://github.com/yogeshsardana/RustShield/actions/workflows/ci.yml/badge.svg)](https://github.com/yogeshsardana/RustShield/actions/workflows/ci.yml)
[![Verus Proofs](https://github.com/yogeshsardana/RustShield/actions/workflows/verus-proofs.yml/badge.svg)](https://github.com/yogeshsardana/RustShield/actions/workflows/verus-proofs.yml)
[![KUnit Tests](https://github.com/yogeshsardana/RustShield/actions/workflows/kunit-tests.yml/badge.svg)](https://github.com/yogeshsardana/RustShield/actions/workflows/kunit-tests.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

---

## Overview

**RustShield** is a framework for safely replacing running C Linux kernel drivers with formally-verified Rust equivalents — **without rebooting, without downtime, without packet loss.**

Memory-safety bugs in kernel drivers account for **>60% of Linux kernel CVEs since 2019**. While Rust-for-Linux (mainline since 6.1) enables writing new drivers in Rust, there has been no safe mechanism to migrate *existing* C drivers to Rust in production. RustShield bridges this gap with a three-phase hotswap protocol.

### The Three-Phase Protocol

```
┌─────────────────────────────────────────────────────────────────────┐
│  PHASE I                      PHASE II                   PHASE III │
│  eBPF Canary                  Verus Proof                Atomic Swap│
│  ┌──────────────┐            ┌──────────────┐          ┌─────────┐ │
│  │ Shadow C     │            │ Formally     │          │ Sub-ms  │ │
│  │ driver IO    │───────────▶│ verify Rust  │─────────▶│ state   │ │
│  │ paths via    │  Baseline  │ replacement  │  Pass    │ handoff │ │
│  │ eBPF probes  │  Spec      │ against spec │          │ via     │ │
│  └──────────────┘            └──────────────┘          │ ioctl   │ │
│                                                         └─────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Quick Start

### Prerequisites

- Linux kernel 6.11+ (for Rust support)
- Rust nightly (toolchain specified in `rust-toolchain.toml`)
- Verus verifier (`cargo install verus`)
- LLVM 18+ with eBPF support
- `bpf-link`, `bpftool` for eBPF canary deployment

### Building

```bash
# Clone and enter
git clone https://github.com/yogeshsardana/RustShield
cd RustShield

# Build the verification kernel module
cd kernel/rust_driver_hotswap && make

# Run Verus proofs
cd ../../verus/verus_kernel_proofs && verus prove src/lib.rs

# Build migration CLI
cd ../../rustshield-migrate && cargo build --release
```

### Running the Demo

See [examples/e1000e_demo/](examples/e1000e_demo/) for the full live migration demo:

```bash
./scripts/demo.sh --driver=e1000e --target=192.168.1.100
```

---

## Repository Structure

```
RustShield/
├── kernel/
│   ├── rust_driver_hotswap/     # Kernel subsystem: DRIVER_HOTSWAP_COMMIT ioctl + DSC protocol
│   ├── bpf_driver_canary/       # eBPF program type for driver behavioral baselines
│   └── driver_migration_tests/  # KUnit-based test harness for behavioral equivalence
├── verus/
│   ├── verus_kernel_proofs/     # Formal proof library: 14 driver safety invariants
│   └── verus_spec/              # Shared specification language for driver behavior
├── rustshield-migrate/          # CLI migration assistant (C → Rust + Verus annotations)
├── examples/
│   ├── e1000e_demo/             # Full live migration demonstration (NIC driver)
│   └── simple_net/              # Simplified example for proof-of-concept
├── docs/
│   ├── architecture.md          # System architecture
│   ├── hotswap-protocol.md      # Three-phase protocol specification
│   ├── proof-library.md         # Verus proof library documentation
│   └── migration-guide.md       # Driver author migration guide
├── rfcs/                        # Upstream RFCs for LKML submission
├── scripts/                     # Demo, benchmark, and dev setup scripts
├── CFP-LSSEU-2026.md            # Linux Security Summit Europe 2026 submission
└── LICENSE                      # Apache 2.0
```

---

## Components

### 1. `rust_driver_hotswap` — Kernel Hotswap Subsystem

Target kernel: **6.11+** | [Documentation](docs/hotswap-protocol.md)

The core kernel subsystem providing:
- `DRIVER_HOTSWAP_COMMIT` ioctl for orchestrating live driver migration
- Device State Capsule (DSC) serialization protocol
- 6-phase atomic state handoff (FREEZE → CAPTURE → VERIFY → TRANSFER → ACTIVATE → COMMIT)
- Sysfs interface for monitoring hotswap status

### 2. `verus_kernel_proofs` — Formal Proof Library

| Documentation](docs/proof-library.md)

The first open corpus of formally verified Linux kernel driver safety invariants:

| Invariant | Description |
|-----------|-------------|
| `interrupt_safety` | No interrupt handler reentrancy |
| `dma_boundary` | DMA buffers never exceed allocated region |
| `refcount_correctness` | Reference count never under/over-flows |
| `lock_ordering` | Locks acquired in consistent global order |
| `device_state_valid` | Device state machine transitions are valid |
| `io_region_exclusive` | No concurrent access to same IO region |
| `timer_safety` | Timers cancelled before module unload |
| `workqueue_ordering` | Work items in correct execution order |
| `dma_mapping_completeness` | All DMA mappings are unmapped |
| `irq_handler_liveness` | IRQ handlers complete in bounded time |
| `register_access_safety` | Device registers accessed with correct width |
| `power_state_transition` | Valid PM transitions |
| `error_recovery` | Error paths leave consistent state |
| `memory_leak_freedom` | No memory leaks on any driver path |

### 3. `bpf_driver_canary` — eBPF Behavioral Oracle

Target kernel: **6.12+** | [Documentation](docs/architecture.md)

A new `BPF_PROG_TYPE_DRIVER_CANARY` eBPF program type that:
- Attaches to C driver kprobes and tracepoints
- Captures IO path behavior, state transitions, and memory access patterns
- Produces a serialized behavioral baseline specification
- Compares runtime behavior of the Rust replacement against the oracle

### 4. `rustshield-migrate` — CLI Migration Assistant

```bash
# Analyze an existing C driver
rustshield-migrate analyze drivers/net/ethernet/intel/e1000e/

# Generate Rust skeleton with Verus annotations
rustshield-migrate skeleton --lang=rust --verus --output=./rust-e1000e/

# Verify against proofs and canary baseline
rustshield-migrate verify --proofs=verus_kernel_proofs --canary=./baseline.json
```

### 5. `RustShield-Nano` — Embedded Hotswap Profile

A constrained profile for embedded Linux (automotive, industrial IoT):
- Targets drivers with ≤ 4 KB device state
- Core proof set (6 invariants)
- Single-phase atomic swap
- ISO 26262 / IEC 61508 compatible

---

## Project Status

RustShield is currently in **active development** and is being prepared for upstream submission to the Linux kernel `linux/rust/` tree.

| Component | Status | Target |
|-----------|--------|--------|
| `rust_driver_hotswap` | Prototype | LKML RFC v1 — Q3 2026 |
| `verus_kernel_proofs` | Proofs under development | v0.1 — Q3 2026 |
| `bpf_driver_canary` | Design phase | Q4 2026 |
| `rustshield-migrate` | CLI skeleton | Alpha — Q3 2026 |
| `e1000e demo` | In progress | LSS EU 2026 demo |

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for guidelines.

### Areas needing help

- Additional Verus proofs for more driver invariants
- eBPF canary program type implementation
- Support for more driver families (virtio, NVMe, GPU)
- RustShield-Nano port for embedded Linux
- Documentation and driver migration guides

---

## License

Apache 2.0 — see [LICENSE](LICENSE).

---

## Related Work

- [Rust-for-Linux](https://rust-for-linux.com/) — Rust support in the Linux kernel
- [Verus](https://github.com/verus-lang/verus) — Formal verification for Rust
- [eBPF](https://ebpf.io/) — Extended Berkeley Packet Filter
- [KUnit](https://kunit.dev/) — Kernel unit testing framework

---

## Acknowledgments

This project builds on the foundational work of the Rust-for-Linux Research, and the eBPF community. The 14 kernel driver invariants draw from the collective experience of the Linux kernel community in identifying and classifying driver-class CVEs.

---
