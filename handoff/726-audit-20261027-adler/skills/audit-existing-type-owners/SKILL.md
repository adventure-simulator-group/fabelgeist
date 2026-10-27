---
name: audit-existing-type-owners
description: "Find primitive fields and APIs that can reuse existing bespoke types. Use for this focused lens or a bespoke-audit assignment."
---

# Existing type owners

Inventory semantic primitives in every scoped record, parameter and return,
then search the relevant repository owners, including unchanged code.

- Compare meaning, units, frame, authority, bounds, valid zero and admission
  policy. Similar names or identical representations do not prove a match.
- Check collections, tuple members, configuration and serialization records;
  a named record can still discard its fields' guarantees.
- Trace the proposed owner through producers and consumers to confirm it fits.
  Identify dependency barriers and the lowest coherent shared location.
- Prefer reuse or a justified owner move over equivalent local wrappers.
  Distinguish an exact match, a reusable numeric leaf with a distinct role,
  a genuinely missing owner and an unresolved contract.

Report concrete matches with both definition and use-site locations. Preserve
accepted deferrals; do not invent an owner or narrow supported values merely
to replace a primitive.

Pin scope/revision; read applicable rules. Before repairs, finish this lens's
exhaustive pass and write an ignored ledger: source locations, contracts/traces,
dispositions, exact connected repairs and acceptance criteria. Account for every
scoped path; group only identical justifications. Untraced paths remain incomplete.
Review only unless fixes/publication are authorized. Distinguish defects,
migration gaps and suggestions.
