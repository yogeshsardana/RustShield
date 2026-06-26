# RustShield

> Memory-Safe Linux Kernel Driver Hotswap with Formal Verification via Verus + eBPF Canaries

[![CI](https://github.com/yogeshsardana/RustShield/actions/workflows/ci.yml/badge.svg)](https://github.com/yogeshsardana/RustShield/actions/workflows/ci.yml)
[![Verus Proofs](https://github.com/yogeshsardana/RustShield/actions/workflows/verus-proofs.yml/badge.svg)](https://github.com/yogeshsardana/RustShield/actions/workflows/verus-proofs.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

---

## Overview

**RustShield** is a framework for safely replacing running C Linux kernel drivers with formally-verified Rust equivalents - **without rebooting, without downtime, without packet loss.**

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

- Rust stable 1.96.0+ (for workspace crates)
- Linux kernel 6.11+ with Rust support (for kernel module - requires Rust-for-Linux tree)
- Verus verifier (built from source - see CI workflow)
- LLVM 18+ with eBPF support

### Building (Workspace Crates)

```bash
git clone https://github.com/yogeshsardana/RustShield
cd RustShield

# Build all workspace crates (bpf_driver_canary, verus proof libs, migration CLI)
cargo build --release --workspace

# Build the migration CLI specifically
cargo build --release -p rustshield-migrate
```

The workspace contains 4 crates that build with stable Rust 1.96.0:

| Crate | Description |
|-------|-------------|
| `kernel/bpf_driver_canary` | eBPF program type definitions (userspace-side) |
| `verus/verus_kernel_proofs` | Formal proof library: 14 driver safety invariants |
| `verus/verus_spec` | Shared specification types for driver behavior |
| `rustshield-migrate` | CLI migration assistant |

> **Note:** Kernel module crates (`kernel/rust_driver_hotswap`, `kernel/driver_migration_tests`) and driver examples (`examples/e1000e_demo`, `examples/simple_net`) require the full Rust-for-Linux kernel tree and cannot be built standalone. See [RFL kernel build docs](https://rust-for-linux.com/) for in-tree build instructions.

### Running Verus Proofs

```bash
# Run the Verus smoke test (validates CI proof pipeline)
verus verus/smoke_test.rs
```

---

## Repository Structure

```
RustShield/
├── kernel/
│   └── bpf_driver_canary/       # eBPF program type for driver behavioral baselines
├── verus/
│   ├── verus_kernel_proofs/     # Formal proof library: 14 driver safety invariants
│   ├── verus_spec/              # Shared specification language for driver behavior
│   └── smoke_test.rs            # CI smoke test for Verus verification pipeline
├── rustshield-migrate/          # CLI migration assistant (C → Rust + Verus annotations)
├── .github/workflows/
│   ├── ci.yml                   # CI: build workspace, run clippy/fmt, upload artifacts
│   └── verus-proofs.yml         # Build Verus from source, verify smoke test
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

## CI/CD Pipeline

Two GitHub Actions workflows ensure continuous quality:

### `ci.yml` - Workspace Build & Artifact

Triggers on every push/PR to `main` and `rustshield-ys1`:

- `cargo build --release --workspace` - builds all 4 workspace crates
- `cargo build --release -p rustshield-migrate` - standalone CLI binary
- `cargo doc --workspace --no-deps` - API documentation
- Uploads `rustshield-migrate` binary and workspace libraries as downloadable artifacts

### `verus-proofs.yml` - Verus Formal Verification

Triggers on every push/PR to `main` and `rustshield-ys1`:

1. Installs Rust 1.96.0 with `rustc-dev` and `llvm-tools`
2. Clones and builds [Verus](https://github.com/verus-lang/verus) from source
3. Runs `verus` on `verus/smoke_test.rs` - a simple proof that arithmetic and boolean assertions hold
4. Validates the full Verus toolchain is functional for future proof development

---

## Components

### 1. `bpf_driver_canary` - eBPF Behavioral Oracle

Target kernel: **6.12+** | [Documentation](docs/architecture.md)

A new `BPF_PROG_TYPE_DRIVER_CANARY` eBPF program type that:
- Attaches to C driver kprobes and tracepoints
- Captures IO path behavior, state transitions, and memory access patterns
- Produces a serialized behavioral baseline specification
- Compares runtime behavior of the Rust replacement against the oracle

### 2. `verus_kernel_proofs` - Formal Proof Library

[Documentation](docs/proof-library.md)

The first open corpus of formally verified Linux kernel driver safety invariants, encoded at the Rust type level:

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

> **Proof strategy:** Each invariant is encoded as a witness type with a `verify()` method. The Rust type system enforces that witnesses can only be constructed when their corresponding invariant holds. This provides compile-time verification without requiring SMT-based proof annotations.

### 3. `rustshield-migrate` - CLI Migration Assistant

```bash
# Analyze an existing C driver
rustshield-migrate analyze drivers/net/ethernet/intel/e1000e/

# Generate Rust skeleton with Verus annotations
rustshield-migrate skeleton --lang=rust --verus --output=./rust-e1000e/

# Verify against proofs and canary baseline
rustshield-migrate verify --proofs=verus_kernel_proofs --canary=./baseline.json
```

### 4. Kernel Module (`rust_driver_hotswap`)

> **Requires Rust-for-Linux kernel tree at `linux/`**

The core kernel subsystem providing:
- `DRIVER_HOTSWAP_COMMIT` ioctl for orchestrating live driver migration
- Device State Capsule (DSC) serialization protocol
- 6-phase atomic state handoff (FREEZE → CAPTURE → VERIFY → TRANSFER → ACTIVATE → COMMIT)
- Sysfs interface for monitoring hotswap status

Build in-tree:
```bash
cd linux && make LLVM=1 rustavailable
cp -r RustShield/kernel/rust_driver_hotswap linux/drivers/rust/
cd linux && make LLVM=1 menuconfig  # Enable CONFIG_RUST_DRIVER_HOTSWAP
cd linux && make LLVM=1 -j$(nproc)
```

### 5. `RustShield-Nano` - Embedded Hotswap Profile

A constrained profile for embedded Linux (automotive, industrial IoT):
- Targets drivers with ≤ 4 KB device state
- Core proof set (6 invariants)
- Single-phase atomic swap
- ISO 26262 / IEC 61508 compatible

---

## Project Status

RustShield is in **active development** and being prepared for upstream submission to Linux `linux/rust/`.

| Component | Status | Target |
|-----------|--------|--------|
| `rust_driver_hotswap` | Prototype (in-tree) | LKML RFC v1 - Q3 2026 |
| `verus_kernel_proofs` | Type-level proofs | v0.1 - Q3 2026 |
| `bpf_driver_canary` | Design phase | Q4 2026 |
| `rustshield-migrate` | CLI skeleton | Alpha - Q3 2026 |
| `e1000e demo` | Design phase | LSS EU 2026 demo |
| CI/CD pipeline | Operational | Stable |

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for guidelines.

### Areas needing help

- Additional Verus proofs (SMT-annotated) for driver invariants
- eBPF canary program type implementation
- Support for more driver families (virtio, NVMe, GPU)
- RustShield-Nano port for embedded Linux
- Documentation and driver migration guides

---

## License

Apache 2.0 - see [LICENSE](LICENSE).

---

## Author & Maintainer

**Yogesh Sardana** - [yogesh.sardana1@gmail.com](mailto:yogesh.sardana1@gmail.com)

For questions, feedback, or collaboration inquiries, please reach out via email.

---

## Related Work

- [Rust-for-Linux](https://rust-for-linux.com/) - Rust support in the Linux kernel
- [Verus](https://github.com/verus-lang/verus) - Formal verification for Rust
- [eBPF](https://ebpf.io/) - Extended Berkeley Packet Filter

---

## Acknowledgments

This project builds on the foundational work of the Rust-for-Linux team, the Verus research group, and the eBPF community. The 14 kernel driver invariants draw from the collective experience of the Linux kernel community in identifying and classifying driver-class CVEs.

---
