---
name: audit-behavior-preservation
description: "Audit unintended behavior changes introduced during a bespoke-type refactor. Use for this focused lens or a bespoke-audit assignment."
---

# Behavior preservation

Compare the pinned base and head across every affected behavior path, including
unchanged callers. Read intended changes and accepted constraints explicitly.

- Compare zero/tiny values, negative and unwrapped coordinates, overflow,
  clamping, normalization, endpoint inclusion and accumulation order.
- Check wire/schema shape, lookup precedence, unknown/duplicate identities,
  error order, missing-data handling, retries and failure-state transitions.
- Inspect conversions and derives for changed defaults, narrowing, sentinel
  handling or silently broadened/restricted accepted input.
- Use relevant existing fixtures/checks within the user's limits. Do not add
  tests, weaken acceptance or infer runtime/visual success from compilation.

Report the old/new path, triggering input and observable difference. Distinguish
an authorized semantic change from a regression. If intended behavior cannot be
established, record the ambiguity instead of assuming preservation.

Pin scope/revision; read applicable rules. Before repairs, finish this lens's
exhaustive pass and write an ignored ledger: source locations, contracts/traces,
dispositions, exact connected repairs and acceptance criteria. Account for every
scoped path; group only identical justifications. Untraced paths remain incomplete.
Review only unless fixes/publication are authorized. Distinguish defects,
migration gaps and suggestions.
