---
name: audit-magic-numbers
description: "Audit every raw non-string primitive and its use of bespoke types. Use for this focused lens or a bespoke-audit assignment."
---

# Magic numbers

Treat every raw non-string primitive as magic. Even `0` and `1` require a
role check.

- Inventory literals, fields, parameters, returns, constants, defaults and
  extracted values across code, configuration, macros, embedded code and fixtures.
- For each occurrence, carry the appropriate existing bespoke type, admit it
  at its actual source boundary, or establish the concrete native operation
  requiring a primitive. Constants and private helpers do not establish a boundary.
- Check units, frames, bounds, sentinels and tolerance error budgets. Preserve
  supported inputs, valid zero and arithmetic order.

Pin scope/revision; read applicable rules. Before repairs, finish this lens's
exhaustive pass and write an ignored ledger: source locations, contracts/traces,
dispositions, exact connected repairs and acceptance criteria. Account for every
scoped path; group only identical justifications. Untraced paths remain incomplete.
Review only unless fixes/publication are authorized. Distinguish defects,
migration gaps and suggestions.
