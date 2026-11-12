---
name: audit-type-granularity
description: "Audit whether bespoke distinctions are too narrow, too broad or redundant. Use for this focused lens or a bespoke-audit assignment."
---

# Type granularity

Inventory affected newtypes, enums, aliases and helper records with their actual
uses. Assess each distinction against the mistakes it prevents.

- Preserve meaningful units, frames, clocks, identity and authority
  distinctions;
  a type can earn its place without stricter numeric validation.
- Find role wrappers that merely duplicate a name with no useful distinction,
  broad validators reused for unrelated quantities, and monolithic records that
  obscure several contracts.
- Share common numeric invariants beneath distinct roles where that reduces
  competing validation. Do not flatten roles just because their storage matches.
- Check whether proposed extra bounds serve a real consumer or introduce an
  unsupported policy. Avoid inventing validation to justify a wrapper.

Show an actual mix-up prevented or a concrete simplification enabled. Prefer
small coherent boundaries over a universal quantity framework. Coordinate owner
placement findings with audit-type-ownership without merging the two passes.

Pin scope/revision; read applicable rules. Before repairs, finish this lens's
exhaustive pass and write an ignored ledger: source locations, contracts/traces,
dispositions, exact connected repairs and acceptance criteria. Account for every
scoped path; group only identical justifications. Untraced paths remain incomplete.
Review only unless fixes/publication are authorized. Distinguish defects,
migration gaps and suggestions.
