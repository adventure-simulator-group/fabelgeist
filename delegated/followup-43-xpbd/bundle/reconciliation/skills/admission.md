---
name: audit-admission-contracts
description: "Audit every way a bespoke value can be constructed or admitted. Use for this focused lens or a bespoke-audit assignment."
---

# Admission contracts

For every affected bespoke type, establish its intended invariant from source,
consumers and requirements, then enumerate all admission routes.

- Inspect constructors, conversions, literals/constants, defaults, builders,
  deserializers, reflection, generated adapters and field visibility.
- Try a representable counterexample: nonfinite input, wrong sign or range,
  invalid sentinel, empty value, wrong provenance, or a bypassing derive.
- Distinguish open identity from validated content. Unfamiliar names need not
  be rejected; finite values need not be clamped, wrapped or normalized.
- Require checked decoding to reach the same authoritative invariant while
  preserving deliberate boundary policies. Label unresolved policies explicitly.

Locate the bypass and smallest coherent admission correction. Arithmetic after
admission belongs to audit-invariant-operations; correlated-field guarantees
belong to audit-aggregate-states. Cross-reference overlaps without skipping
this lens's coverage.

Pin scope/revision; read applicable rules. Before repairs, finish this lens's
exhaustive pass and write an ignored ledger: source locations, contracts/traces,
dispositions, exact connected repairs and acceptance criteria. Account for every
scoped path; group only identical justifications. Untraced paths remain incomplete.
Review only unless fixes/publication are authorized. Distinguish defects,
migration gaps and suggestions.
