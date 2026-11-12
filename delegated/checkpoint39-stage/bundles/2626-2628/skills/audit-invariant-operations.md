---
name: audit-invariant-operations
description: "Audit arithmetic and mutation that can break admitted bespoke-value guarantees. Use for this focused lens or a bespoke-audit assignment."
---

# Invariant-preserving operations

Enumerate operations producing or replacing affected values: arithmetic traits,
accumulation, interpolation, scaling, normalization, indexing and mutation.

- Trace input guarantees to the published result. Finite operands can overflow;
  bounded operands need not produce bounded sums, norms or ratios.
- Check zero denominators, integer overflow/underflow, narrowing, ordering,
  endpoints and intentional wrapping/saturation. Require an explicit policy.
- Follow failure paths through retained state. Rejecting an operation must not
  accidentally commit part of one logical update or consume its retry clock.
- Inspect unchecked internal construction and native kernels as well as public
  methods. Re-admit computed results where the arithmetic loses the guarantee.

Report a concrete reachable case or representable API counterexample, with the
operation and affected consumer. Distinguish current failures from contract
strengthening; do not require infallible arithmetic where rejection is valid.

Pin scope/revision; read applicable rules. Before repairs, finish this lens's
exhaustive pass and write an ignored ledger: source locations, contracts/traces,
dispositions, exact connected repairs and acceptance criteria. Account for every
scoped path; group only identical justifications. Untraced paths remain incomplete.
Review only unless fixes/publication are authorized. Distinguish defects,
migration gaps and suggestions.
