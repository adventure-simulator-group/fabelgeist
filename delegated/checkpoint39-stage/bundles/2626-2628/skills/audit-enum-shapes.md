---
name: audit-enum-shapes
description: "Audit semantic booleans and enums that should own neighboring payloads. Use for this focused lens or a bespoke-audit assignment."
---

# Enum shapes

Inventory affected carried booleans, flags, tags, optional payloads and enums.
Inspect construction, transitions, consumers and serialization together.

- Distinguish a semantic choice carried between APIs from an ordinary named
  predicate or required native flag.
- Find tags beside conditionally meaningful fields and correlated booleans whose
  combinations are invalid. Consider variants that carry only their valid data.
- If initialization is wholly represented by optional history, assess removing
  the redundant flag rather than mechanically creating another enum.
- Keep independent axes separate. Preserve meaningful inactive payloads and
  historical observations rather than assuming they disappear on transition.
- Keep Boolean wire encoding at its real adapter; serialization cannot infer
  which semantic variant means true.

Propose the concrete state shape and explain the combinations it prevents.
Reuse an existing enum where it means the same thing; avoid enums that merely
rename arbitrary numeric indices.

Pin scope/revision; read applicable rules. Before repairs, finish this lens's
exhaustive pass and write an ignored ledger: source locations, contracts/traces,
dispositions, exact connected repairs and acceptance criteria. Account for every
scoped path; group only identical justifications. Untraced paths remain incomplete.
Review only unless fixes/publication are authorized. Distinguish defects,
migration gaps and suggestions.
