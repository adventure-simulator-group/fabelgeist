---
name: audit-derivability
description: "Audit unnecessary handwritten trait and serialization boilerplate without weakening contracts. Use for this focused lens or a bespoke-audit assignment."
---

# Derivability

Inventory affected handwritten trait implementations and compare available
project derives against their exact semantics and bounds.

- Separate mechanical field projection/delegation from validation, semantic
  classification, checked narrowing and representation changes.
- Check generic bounds, borrowed versus owned output, visibility, error type,
  native-index exposure and wire shape before recommending a derive.
- Checked deserialization may derive through validated conversion. A transparent
  derive or raw construction derive can bypass admission.
- A named borrowed serialization view can derive mechanical record writing;
  retain the explicit semantic mapping that constructs that view.

Name the existing derive/dependency and equivalent implementation. Check project
source or primary documentation when uncertain. Do not add serialization merely
to replace a runtime classifier, or prescribe a derive that changes the API.
Keep necessary small manual implementations.

Pin scope/revision; read applicable rules. Before repairs, finish this lens's
exhaustive pass and write an ignored ledger: source locations, contracts/traces,
dispositions, exact connected repairs and acceptance criteria. Account for every
scoped path; group only identical justifications. Untraced paths remain incomplete.
Review only unless fixes/publication are authorized. Distinguish defects,
migration gaps and suggestions.
