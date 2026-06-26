# RustShield Hotswap Protocol — Specification

## 1. Terminology

| Term | Definition |
|------|------------|
| C Driver | The existing kernel driver written in C that is to be replaced |
| Rust Driver | The replacement driver written in Rust and verified with Verus |
| DSC | Device State Capsule — serialized snapshot of device state |
| Canary | eBPF program that shadows C driver behavior to produce a baseline |
| Oracle | Comparison engine that checks Rust driver behavior against baseline |
| Hotswap Window | The time interval (≤ 1 ms) during which the device is locked |
| COMMIT | The final ioctl that completes the hotswap atomically |

## 2. Protocol Phases

The hotswap protocol consists of 6 phases, executed in sequence by the `DRIVER_HOTSWAP_COMMIT` ioctl handler:

### Phase 1: FREEZE

**Goal**: Quiesce the C driver so no new IO operations are in flight.

**Steps**:
1. Disable device interrupts via the IRQ controller
2. Mask all MSI-X vectors for the device
3. Drain TX completion rings (wait for all pending descriptors)
4. Drain RX rings (return all buffers to the network stack)
5. Wait for DMA operations to complete (check DMA completion rings)
6. Flush workqueues associated with the driver
7. Flush tasklets and softirq handlers
8. Verify: count of pending operations == 0

**Timeout**: 500 µs. On timeout, ROLLBACK is triggered.

### Phase 2: CAPTURE

**Goal**: Serialize all device state into a Device State Capsule.

**Steps**:
1. Read all MMIO register ranges (stored in `pci_resource_start/len`)
2. Read PIO port ranges if applicable
3. Snapshot DMA mapping table (IOMMU page tables if applicable)
4. Capture IRQ state vector (pending bits, enabled/disabled)
5. Serialize software state:
   - TX/RX descriptor ring pointers and contents
   - Packet buffer addresses and lengths
   - Device statistics counters
   - Driver private data structures
6. Compute DSC checksum (CRC64 or similar)

**Output**: `DeviceStateCapsule` struct.

### Phase 3: VERIFY

**Goal**: Validate the captured state and confirm the Rust driver is ready.

**Steps**:
1. Verify DSC checksum integrity
2. Validate DSC protocol version
3. Check all state region boundaries (no overflow)
4. Verify against eBPF canary baseline:
   - Compare register values against baseline oracle
   - Compare state machine state against expected transitions
   - Verify DMA region usage matches baseline
5. Check that Rust driver's Verus proofs are current (not stale)
6. Check that Rust module is loaded and its init function succeeded

**On failure**: ROLLBACK is triggered with detailed error reporting.

### Phase 4: TRANSFER

**Goal**: Atomically switch from C driver to Rust driver.

**Steps**:
1. Take device-specific lock (mutex/spinlock)
2. Override driver struct ops:
   - Replace `pci_driver` or `platform_driver` ops with Rust equivalents
   - Replace `net_device_ops` or `block_device_ops`
   - Replace `ethtool_ops` if applicable
3. Restore device registers from DSC (write captured values back)
4. Restore DMA mapping table
5. Restore IRQ handler:
   - Disassociate C IRQ handler
   - Associate Rust IRQ handler
6. Restore software state:
   - Re-initialize descriptor rings from capsule
   - Re-populate DMA buffers from capsule
   - Restore private data structures

**Atomicity**: All steps within TRANSFER are performed under a single spinlock with preemption disabled. The total time must be ≤ 100 µs.

### Phase 5: ACTIVATE

**Goal**: Start the Rust driver and verify correct operation.

**Steps**:
1. Enable device interrupts (Rust handler now active)
2. Enable MSI-X vectors
3. Resume TX/RX rings (Rust driver's ring management)
4. Start watchdog timer (for the Rust driver)
5. Begin sampling eBPF canary events from Rust driver
6. Compare Rust canary events against baseline oracle
7. Verify agreement ratio ≥ threshold (default 99.9%)

**On canary mismatch**: ROLLBACK is triggered.

### Phase 6: COMMIT

**Goal**: Finalize the hotswap as successful.

**Steps**:
1. Mark C driver for deferred unloading (scheduled work)
2. Notify sysfs listeners (`/sys/kernel/rustshield/status` → "completed")
3. Log hotswap event to kernel ring buffer:
   ```
   rustshield: Hotswap COMMIT: e1000e -> rust-e1000e
               (0 packet loss, 1.2 ms total window)
   ```
4. Return success from `DRIVER_HOTSWAP_COMMIT` ioctl
5. Unload C module via async workqueue

## 3. Device State Capsule (DSC) Format

```
┌──────────────────────────────────────┐
│ protocol_version: u8 = 1             │
│ driver_name: [u8; 64]                │
│ device_bus_info: [u8; 64]            │
│ state_regions: [StateRegion; N]      │
│ irq_state: IrqStateSnapshot          │
│ dma_state: DmaStateSnapshot          │
│ refcounts: [(String, u64); M]        │
│ lock_state: LockStateSnapshot        │
│ timestamp_ns: u64                    │
│ checksum: u64                        │
└──────────────────────────────────────┘

StateRegion:
├── offset: u64
├── size: u64
├── kind: enum { MMIO, PIO, DMA, Register, ConfigSpace, PrivateData }
└── data: [u8; size]
```

Maximum total size: 4096 bytes (configurable via `RUST_DRIVER_HOTSWAP_MAX_DSC_SIZE`).

## 4. ioctl Interface

```c
#define DRIVER_HOTSWAP_COMMIT    _IOWR('R', 1, struct hotswap_args)
#define DRIVER_HOTSWAP_STATUS    _IOR('R', 2, struct hotswap_status)
#define DRIVER_HOTSWAP_ROLLBACK  _IO('R', 3)

struct hotswap_args {
    char driver_name[64];       // C driver to replace
    char rust_module[256];      // Rust module path
    __u32 flags;                // flags (FORCE, DRY_RUN, SKIP_VERIFY, etc.)
    __u8  dsc_buffer[4096];     // output: captured device state capsule
};

struct hotswap_status {
    __u32 phase;                // current hotswap phase
    __u32 canary_agreement_pct; // 0-100
    __u64 last_hotswap_ns;      // timestamp of last hotswap
    __u32 hotswap_count;        // total successful hotswaps
    __u32 rollback_count;       // total rollbacks
};
```

## 5. Flag Definitions

| Flag | Value | Description |
|------|-------|-------------|
| `FORCE` | 0x01 | Skip canary verification |
| `DRY_RUN` | 0x02 | Execute all phases except COMMIT |
| `SKIP_VERIFY` | 0x04 | Use cached proof results |
| `VERBOSE` | 0x08 | Verbose kernel logging |
| `NANO` | 0x10 | Use RustShield-Nano single-phase profile |

## 6. Error Codes

| Error | Code | Meaning |
|-------|------|---------|
| `EBUSY` | 16 | Another hotswap in progress |
| `ENODEV` | 19 | Driver not found |
| `EINVAL` | 22 | Invalid argument or DSC |
| `ETIMEDOUT` | 110 | Hotswap window exceeded |
| `EAGAIN` | 11 | Canary mismatch, retry recommended |
| `EPERM` | 1 | Permission denied (missing CAP_SYS_ADMIN) |
| `EOPNOTSUPP` | 95 | Driver family not supported |

## 7. Locking and Concurrency

- Only one hotswap operation may be in progress system-wide (enforced by atomic flag)
- During TRANSFER phase, the device's main lock is held with preemption disabled
- IRQs are masked during the hotswap window
- The C driver's module reference count is held until COMMIT completes
- The Rust driver is loaded before FREEZE begins (to minimize the locked window)
