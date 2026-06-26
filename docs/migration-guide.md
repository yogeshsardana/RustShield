# RustShield Migration Guide

## For Driver Authors: Converting a C Driver to a Verified Rust Driver

### Overview

This guide describes the process of migrating an existing C kernel driver to a formally verified Rust driver using RustShield.

### Prerequisites

- Rust nightly (toolchain in `rust-toolchain.toml`)
- Verus verifier installed (`cargo install verus`)
- Linux kernel source with Rust support (6.11+)
- `bpftool` and `bpf-link` for eBPF canary deployment
- The C driver source code you want to migrate

### Step 1: Analyze the C Driver

```bash
rustshield-migrate analyze drivers/net/ethernet/intel/e1000e/ \
    --output e1000e-analysis.json
```

This produces:
- Function count and call graph
- IO operation inventory (readl/writel, ioread/iowrite)
- State variable identification and size estimation
- Lock analysis (mutexes, spinlocks, their ordering)
- DMA region identification
- Migration readiness score (0-100)

### Step 2: Generate Rust Skeleton

```bash
rustshield-migrate skeleton \
    --analysis e1000e-analysis.json \
    --verus \
    --output ./rust-e1000e/
```

This generates:
- `Cargo.toml` with kernel and proof library dependencies
- `src/lib.rs` with:
  - Device data struct
  - State enum
  - Function stubs matching all NDO operations
  - Verus trait implementations (14 invariants)
  - Placeholder annotations for each proof

### Step 3: Implement the Rust Driver

Fill in each function stub:

```rust
// Before (generated stub):
pub fn probe(pdev: &mut kernel::platform::Device) -> Result<Box<DeviceData>> {
    Err(ENODEV)
}

// After (implemented):
pub fn probe(pdev: &mut kernel::platform::Device) -> Result<Box<DeviceData>> {
    let regs = kernel::io::IoMem::try_new(pdev)?;
    let irq = pdev.irq_number()?;
    let dev = Box::try_new(DeviceData {
        regs: Some(regs),
        irq,
        state: Mutex::new(DeviceState::Down),
    })?;
    request_irq(irq, interrupt_handler, 0, "rust-e1000e", dev.as_ref())?;
    Ok(dev)
}
```

### Step 4: Add Verus Annotations

Annotate each function with pre/post conditions:

```rust
#[verus::requires(dev.state.get() == DeviceState::Down)]
#[verus::ensures(|result: Result<()>| result.is_ok() ==> dev.state.get() == DeviceState::Up)]
pub fn open(dev: &mut DeviceData) -> Result {
    let mut state = dev.state.lock();
    *state = DeviceState::Up;
    Ok(())
}
```

### Step 5: Run Verus Proofs

```bash
verus prove src/lib.rs \
    --library ../../verus/verus_kernel_proofs
```

Expected output:
```
Proof verification: 14/14 invariants verified ✓
```

### Step 6: Deploy eBPF Canary Baseline

```bash
# On the production system running the C driver:
bpftool canary deploy \
    --driver e1000e \
    --output canary-baseline.json

# Duration: 5 minutes (configurable)
# Captures typical and peak behavior patterns
```

### Step 7: Load Rust Driver and Verify

```bash
# Load the Rust driver (inactive mode):
insmod rust-e1000e.ko

# Run verification against canary:
rustshield-migrate verify \
    --proofs ../../verus/verus_kernel_proofs \
    --canary canary-baseline.json \
    --driver ./rust-e1000e

# Output:
# ✓ Verus proofs: All 14 invariants verified
# ✓ Canary agreement: 100.0%
# ✓ Readiness score: 100/100
```

### Step 8: Execute Hotswap

```bash
# Trigger the live migration:
./scripts/demo.sh --driver=e1000e --rust-module=rust-e1000e

# Or manually via ioctl:
./rustshield-ctl hotswap \
    --driver e1000e \
    --rust-module rust-e1000e
```

Monitor via sysfs:
```bash
cat /sys/kernel/rustshield/status
# → "completed"
```

### Step 9: Verify Production

```bash
# Check traffic:
ethtool -S eth0
# Compare counters — should be identical pre/post migration

# Check no packet loss:
netstat -s | grep -i loss
# → 0 packet loss

# Check driver in use:
ethtool -i eth0
# → driver: rust-e1000e
```

## RustShield-Nano (Embedded Profile)

For embedded Linux with constrained resources:

```bash
rustshield-migrate migrate \
    --path drivers/i2c/busses/i2c-designware/ \
    --nano \
    --output ./rust-i2c-designware
```

The Nano profile:
- Targets drivers with ≤ 4 KB device state
- Verifies 6 core invariants (interrupt_safety, dma_boundary, refcount, lock_ordering, device_state, memory_leak)
- Skips eBPF canary baseline (single-phase swap)
- Complies with ISO 26262 ASIL-D / IEC 61508 SIL-3

## Troubleshooting

### Proof Verification Fails

```bash
verus prove src/lib.rs --explain
```

Common issues:
- Missing or incorrect pre/post conditions on functions
- Lock ordering not declared in `LockGraph`
- State transitions don't match the specification
- DMA regions not declared in `DmaBoundaryWitness`

### Canary Agreement < 100%

```bash
bpftool canary compare \
    --baseline canary-baseline.json \
    --rust-events /sys/kernel/debug/rustshield/canary_events
```

Common causes:
- Timing differences (relax thresholds with `--tolerance=0.05`)
- Different hardware revision (re-run baseline on target hardware)
- Kernel config differences (ACPI vs DT, MSI vs MSI-X)

### Hotswap Fails

Check kernel logs:
```bash
dmesg | grep rustshield
```

Common causes:
- Device busy with in-flight operations (increase FREEZE timeout)
- DSC exceeds maximum size (increase `RUST_DRIVER_HOTSWAP_MAX_DSC_SIZE`)
- IRQ handler registration conflict (check IRQ affinity)
