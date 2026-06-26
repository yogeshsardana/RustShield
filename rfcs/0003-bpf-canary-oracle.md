# RFC 0003: eBPF Canary Program Type for Driver Behavioral Oracles

- Feature Name: `bpf_driver_canary`
- Start Date: 2026-03-15
- RFC PR: TBD

## Summary

Introduce a new eBPF program type (`BPF_PROG_TYPE_DRIVER_CANARY`) for shadowing C driver critical paths and producing behavioral baselines that guide and verify Rust driver migration.

## Motivation

To safely replace a C driver with a Rust equivalent, we must first understand the C driver's behavioral contract. Existing eBPF program types (kprobe, tracepoint, perf_event) can observe individual events but lack the structured program context needed to produce a driver behavioral specification.

## Guide-Level Explanation

The new program type provides:
- Canary attach points for all driver operations (probe, open, xmit, IRQ, DMA, register IO)
- Structured event format with driver-specific context
- Ring buffer for zero-copy event collection
- Baseline serialization to a behavioral specification format

## Reference-Level Explanation

See `kernel/bpf_driver_canary/src/lib.rs` for the program type definition.

Key attach points:
- `BPF_CANARY_ATTACH_PROBE_ENTRY/EXIT`
- `BPF_CANARY_ATTACH_OPEN_ENTRY/EXIT`
- `BPF_CANARY_ATTACH_XMIT_ENTRY/EXIT`
- `BPF_CANARY_ATTACH_IRQ_ENTRY/EXIT`
- `BPF_CANARY_ATTACH_DMA_ALLOC/FREE`
- `BPF_CANARY_ATTACH_REG_READ/WRITE`
- `BPF_CANARY_ATTACH_STATE_TRANSITION`

## Drawbacks

- New eBPF program type requires BPF subsystem review
- Canary overhead adds ~1-3% performance impact during baseline collection
- Behavioral baseline is hardware-specific (different revision → different baseline)

## Rationale and Alternatives

- Alternative: Manual specification — error-prone, incomplete
- Alternative: Static analysis of C source — misses runtime behavior
- Alternative: No baseline — blind migration risk

## Unresolved Questions

- Should canary data include full register traces or just signatures?
- How long should baseline collection run to capture representative behavior?
- Should canary probes be deployable on production systems?

## Future Possibilities

- Continuous canary monitoring for Rust drivers (regression detection)
- Automated baseline generation from driver test suites
- Cross-architecture baseline comparison (x86 vs ARM)
