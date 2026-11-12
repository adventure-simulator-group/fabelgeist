---
name: audit-redundant-authority
description: "Audit independently stored or recomputed sources of truth for one fact. Use for this focused lens or a bespoke-audit assignment."
---

# Redundant authority

Inventory affected authoritative values, cached classifications, flags,
collections, defaults and policy tables. Trace every construction/update path.

- Find fields derivable entirely from neighboring state, cached labels that
  survive independent identity mutation, and consumers that reclassify text.
- Compare configuration defaults and copies with the actual authored owner.
  Equality tests detect drift; they do not remove duplicate authority.
- Check cache invalidation against every dependency, including asset revisions,
  shape/config changes and failed replacement. Caching itself is not a defect.
- Distinguish independent observations, temporal history and orthogonal facts
  from competing representations of the same current fact.

Show how the representations can disagree and which consumer observes it.
Prefer deriving the value, one coordinated update, or explicit cache ownership;
do not collapse independent state merely because it often changes together.

Pin scope/revision; read applicable rules. Before repairs, finish this lens's
exhaustive pass and write an ignored ledger: source locations, contracts/traces,
dispositions, exact connected repairs and acceptance criteria. Account for every
scoped path; group only identical justifications. Untraced paths remain incomplete.
Review only unless fixes/publication are authorized. Distinguish defects,
migration gaps and suggestions.
