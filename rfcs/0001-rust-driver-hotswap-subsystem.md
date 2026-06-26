# RFC 0001: Rust Driver Hotswap Subsystem

- Feature Name: `rust_driver_hotswap`
- Start Date: 2026-03-15
- RFC PR: TBD
- Linux Kernel Issue: TBD

## Summary

Introduce a new kernel subsystem under `linux/rust/` that provides a `DRIVER_HOTSWAP_COMMIT` ioctl for safely replacing a running C kernel driver with a Rust equivalent without rebooting.

## Motivation

Memory-safety bugs in kernel drivers account for >60% of kernel CVEs since 2019. While the Rust-for-Linux effort (mainline since 6.1) enables writing new drivers in Rust, there is no mechanism to migrate existing C drivers to Rust in production without a maintenance window and reboot.

## Guide-Level Explanation

The subsystem provides:

1. **Character device** (`/dev/rustshield`) with ioctl interface for triggering hotswap
2. **Device State Capsule** serialization format for capturing and restoring driver state
3. **6-phase protocol** (FREEZE → CAPTURE → VERIFY → TRANSFER → ACTIVATE → COMMIT)
4. **Sysfs interface** at `/sys/kernel/rustshield/` for monitoring

## Reference-Level Explanation

See `docs/hotswap-protocol.md` for the full protocol specification.

Key design decisions:
- Sub-millisecond hotswap window with IRQs masked
- Rollback support from any phase before COMMIT
- DSC format versioned for forward compatibility
- Single global lock ensures only one hotswap at a time

## Drawbacks

- Additional complexity in the driver core
- Only works with drivers that support the DSC serialization protocol
- Requires Rust module to be pre-loaded (increases memory footprint)

## Rationale and Alternatives

- Alternative: `kexec` + reboot — unacceptable downtime
- Alternative: Module stacking — not supported by kernel module loader
- Alternative: Live patching (KGraft) — function-level, no state migration

## Unresolved Questions

- Should DSC format be device-tree-like or binary?
- How to handle drivers with hardware-dependent state entropy?
- Should we support hotswap between two C drivers (C→C)?

## Future Possibilities

- C→C hotswap for emergency CVE patching without Rust
- Userspace-driven DMA region reconfiguration
- Automated canary deployment via udev rules
