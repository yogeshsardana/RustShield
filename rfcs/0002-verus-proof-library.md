# RFC 0002: Verus Proof Library for Kernel Driver Invariants

- Feature Name: `verus_kernel_proofs`
- Start Date: 2026-03-15
- RFC PR: TBD

## Summary

Introduce a Verus proof library (`verus_kernel_proofs`) that formalizes 14 Linux kernel driver safety invariants, enabling compile-time verification of Rust drivers against these invariants.

## Motivation

Current Rust-for-Linux drivers can still contain logic bugs (as opposed to memory safety bugs). Formal verification with Verus enables proving the absence of entire classes of driver bugs at compile time, complementing Rust's type system guarantees.

## Guide-Level Explanation

Driver authors implement the `DriverSafetyContract` trait, which requires providing proof witnesses for each invariant. Verus checks these proofs at compile time. Drivers that pass all 14 proofs are guaranteed to be free of:
- IRQ reentrancy bugs
- DMA buffer overflows
- Reference count errors
- Deadlocks from inconsistent lock ordering
- And 11 other driver bug classes

## Reference-Level Explanation

See `docs/proof-library.md` for the full proof module documentation.

Each proof module uses Verus's ghost state and refinement type system to encode invariants. Proofs are zero-cost at runtime.

## Drawbacks

- Increases compile time by 5-15%
- Requires driver authors to learn Verus annotation syntax
- Some invariants may be undecidable for complex drivers

## Rationale and Alternatives

- Alternative: Runtime assertion checks — miss bugs in non-executed paths
- Alternative: Model checking (CBMC) — external tool, no Rust integration
- Alternative: No verification — status quo, >60% CVEs from drivers

## Unresolved Questions

- Should proof library be in `linux/rust/` or a separate repository?
- How to handle driver-specific invariants beyond the 14 core ones?
- Should proofs be re-run at module load time (for supply chain security)?

## Future Possibilities

- Automated proof synthesis from C driver analysis
- Proof-aware fuzzing (Verus + libFuzzer integration)
- SMT-based proof optimization for faster compilation
