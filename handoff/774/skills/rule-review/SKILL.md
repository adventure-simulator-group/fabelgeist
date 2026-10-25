---
name: rule-review
description: Review a change or codebase against its governing repository rules, including bespoke types, API ownership, module boundaries, and enforcement gaps. Use for requested rule or style audits and bespoke-type migration reviews.
---

# Rule review

Establish the review scope from the user's request: working changes, a supplied
revision range, or the codebase. A review request authorizes inspection and
verification; implement fixes only when requested. Do not publish comments or
change issues as a side effect of a local review.

Read the applicable `AGENTS.md` files and any specifically named rule sources.
Treat rules as scoped contracts, not a universal style preference. Distinguish
requirements from recommendations and accepted exceptions. Inspect code and
behavioral tests rather than trusting issue closure or a passing lint gate.

For a bespoke-type migration, or an explicit semantic-contract review, also read
[references/bespoke-type-review.md](references/bespoke-type-review.md). Apply
that mode to the claimed and directly affected semantic families. A changed type
definition is not the review boundary: inspect its producers, stored records,
operations and consumers, including unchanged receiving code. Report migration
completeness separately from correctness; earlier verdicts do not establish
fresh source coverage.

Review the contracts relevant to the changed responsibilities:

- Follow a semantic value from its authoritative owner through serialization and
  its consumers. Check that constructors, deserialization, reflection, and
  adapters preserve its invariants. Reuse existing types instead of adding
  parallel wrappers; keep distinct authority domains separate.
- Look for behavior driven by display prose, serialized status strings, or
  Debug output. Include iterator predicates, guards, and expression macros.
  Distinguish closed vocabularies from actual text, catalog IDs, format tokens,
  and external protocols.
- Inspect numeric operations for units, scales, and bounds. A one-use literal
  can carry domain meaning; repetition alone does not make a literal wrong.
  Check whether a named value or type already owns the meaning.
- Assess modules by responsibility and visibility. File splits, includes,
  section banners, and pass-through wrappers do not by themselves establish
  ownership. Check methods and constructors against the repository's API rules.
- Inspect suppressions and baselines. Separate grandfathered debt from new
  regressions. Compare changed ceilings with the base revision; a regenerated
  baseline can conceal growth. Do not treat baseline entries as independently
  verified violations.

Run the relevant existing checks. When checking an enforcement tool, use small
isolated positive and negative probes that exercise ordinary language forms;
a successful repository scan alone does not demonstrate coverage. Keep evidence
in the repository's ignored output location. Do not fix a gate by weakening its
contract, raising ceilings, or broadly exempting production code.

Report actionable findings with severity, exact location, governing rule,
concrete consequence, and a proportionate correction. State verification and
coverage limits, including unavailable tools. Exclude unconfirmed speculation,
line-count-only complaints where cohesion is allowed, and generated-code edits
that belong in the owning schema or generator. If no actionable gap is found,
say so and state the scope checked.
