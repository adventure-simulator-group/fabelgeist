---
name: comment-review
description: Review source-code comments and API documentation for accuracy, useful intent, stale claims, and consistency with behavior. Use for requested comment-quality reviews; not for responding to pull-request discussion threads.
---

# Comment review

Review source comments and documentation within the requested change or area.
Read the governing repository and editorial rules first. Follow the local
voice; do not invent a universal ban on comments or replace useful explanations
with terseness. Review-only requests do not authorize rewriting or publishing.

Read each important comment with its code, callers, and relevant tests. Check:

- Does it describe the current contract, units, ownership, failure behavior,
  or reason for a non-obvious decision accurately?
- Do documented invariants hold at every construction and boundary path?
- Does it explain a constraint or tradeoff that the code cannot show clearly,
  or merely narrate the next statement and add maintenance burden?
- Does it retain obsolete implementation history, temporary completion notes,
  unsupported provenance, or claims about safety or performance without
  evidence? A historical note is useful when it explains a current constraint.
- Do examples, parameter descriptions, links, and TODOs still match the API?
  Verify external factual claims when needed; never invent citations.
- Does the comment expose an architectural problem rather than solve it?
  Section banners and ownership claims do not create module boundaries.

Preserve license notices, required attribution, safety contracts, generated
markers, and explanations of unintuitive behavior. Distinguish source comments
from user-visible copy and from comments maintained by a generator. Apply any
separate documentation approval workflow only to the documents it governs.

For each finding, give an exact location and explain the mismatch or unnecessary
burden. Suggest precise replacement wording when the intended contract is
established. If the code and comment disagree and intent is unclear, identify
the ambiguity instead of silently choosing a new behavior. If fixes are
requested, make focused edits and run formatting, documentation examples, or
other checks affected by those edits; avoid unrelated code rewrites.

Report remaining uncertainties and what was checked. Prefer a few substantiated
findings over a catalog of subjective wording preferences.
