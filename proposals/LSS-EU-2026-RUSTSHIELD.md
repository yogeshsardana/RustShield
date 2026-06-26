# RustShield: Memory-Safe Linux Kernel Driver Hotswap with Formal Verification via Verus + eBPF Canaries

**Proposal for Linux Security Summit Europe 2026 — Prague, Czechia**

---

## Session Format

- **Type:** Technical Talk + Live Demonstration (45 minutes)
- **Track:** Linux Kernel Security & Systems Programming
- **Audience Level:** Intermediate to Advanced

---

## Abstract

Memory-safety bugs in Linux kernel drivers account for over 60% of kernel CVEs since 2019. While the Linux kernel has begun accepting Rust drivers (mainline since 6.1), there is no mechanism to safely replace a running C driver with a Rust equivalent without rebooting — a critical gap for production systems requiring continuous availability.

RustShield introduces a first-of-its-kind kernel driver live-migration framework that safely replaces a running C kernel driver with a formally-verified Rust equivalent using a three-phase hotswap protocol: (1) eBPF canaries shadow the C driver's critical paths to establish a behavioral baseline, (2) the Rust replacement is formally verified against the baseline using Verus, and (3) a kernel-managed state handoff transfers device state atomically using a new ioctl DRIVER_HOTSWAP_COMMIT. RustShield also contributes a Verus proof library covering 14 common Linux driver safety invariants, enabling driver authors to formally verify their Rust drivers with minimal annotation overhead.

Live demo: replacing a running e1000e NIC driver with a Rust equivalent on a live VM — zero packet loss, zero reboot.

---

## Why This Matters

The Linux kernel Rust effort is making progress, but the migration path from C to Rust drivers is entirely offline — drivers must be replaced during a maintenance window, limiting adoption in critical production infrastructure. RustShield's contribution is transformational:

- **For the kernel Rust subsystem:** Four upstream-targetable components to linux/rust/
- **For infrastructure operators:** Rolling Rust driver migration in production without downtime
- **For formal verification research:** First open corpus of verified Linux driver safety invariants
- **For CNCF/container runtimes:** Live driver migration in Kubernetes without node drain
- **For IoT/embedded:** RustShield-Nano profile for ISO 26262 / IEC 61508 systems
- **For driver authors:** rustshield-migrate CLI compresses C-to-Rust migration from weeks to days

---

## Speaker Bio

*[Submitter to provide]*

---

## Technical Requirements for Demo

- Laptop with QEMU/KVM and a VM running Linux 6.11+ with e1000e NIC
- Projector connection (HDMI)
- Internet access (for live repository cloning, optional)

---

## Submitted To

Linux Security Summit Europe 2026
Prague, Czechia
