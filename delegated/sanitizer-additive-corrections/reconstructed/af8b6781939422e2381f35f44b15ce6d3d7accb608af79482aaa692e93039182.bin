---
name: bespoke-audit
description: "Coordinate exhaustive bespoke-type and semantic review waves using thirteen focused audit skills. Use when requested to run the suite, simulate repeated bespoke-type review, or review and repair until all selected lenses clear."
---

# Bespoke audit waves

Use every lens below unless the user selects a subset. Load and run one lens at
a time; overlapping findings do not substitute for another lens's coverage.

1. Pin base/head and requested working changes; fingerprint uncommitted files.
   Inventory every changed file, including tests, configuration and
   generated-source owners, plus affected unchanged paths. Read applicable rules
   and explicit deferrals; do not sample promising families.
2. Before any repair, finish exhaustive discovery for every selected lens and
   write one ignored ledger: lens, source locations/contract or trace,
   evidence/disposition, exact connected repair/dependencies, acceptance criterion,
   and separate discovery/repair/validation statuses. Start discovery **incomplete**.
   Enumerate files and account for intentional, deferred and non-applicable
   candidates with reasons. Search hits and prior verdicts are not coverage.
3. Deduplicate by defect/remedy against existing discussion, preserving all lens
   support and locations. Reconcile shared-function repairs and order dependencies.
   Distinguish defects, migration gaps, strengthening and accepted follow-ups;
   cite rules. Record blockers/policy questions and continue independent discovery.
4. Review-only returns the ledger/findings. Repairs, new tests, publication and
   delegation require authorization. Implement from the ledger; extend it before
   following new paths. After planned repairs, run another complete wave at the
   new head, revalidating carried evidence. Targeted rechecks are not full waves.
5. **Clear** requires complete coverage of every selected lens at the same current
   snapshot with no actionable unresolved findings. Explicit user deferrals allow
   **clear within scope, with deferrals**, naming their authority. Unread paths,
   blockers and open policies remain **incomplete**. CI and zero new comments do
   not establish clearance. Reconfirm head/fingerprints before clearance or
   publication.

On interruption, save revision, ledger and first unfinished cell; resume there.
Work sequentially unless delegation is permitted and requested. Preserve pending
reviews; do not submit without authorization or label self-review independent.

| Lens | Ledger inventory/trace |
| --- | --- |
| [Unfinished producers](../audit-producer-propagation/SKILL.md) | Producer → state → operations → recipient → boundary |
| [Existing owners](../audit-existing-type-owners/SKILL.md) | Primitive use → candidate owner → contract fit/dependencies |
| [Admission](../audit-admission-contracts/SKILL.md) | Invariant → all admission routes → bypass/counterexample |
| [Operations](../audit-invariant-operations/SKILL.md) | Input guarantees → operation → result/failure → committed state |
| [Aggregates](../audit-aggregate-states/SKILL.md) | Field relationship → construction/mutation → relying consumer |
| [Wrong/duplicate owners](../audit-type-ownership/SKILL.md) | Concept → definitions/users/dependencies → canonical owner |
| [Redundant authority](../audit-redundant-authority/SKILL.md) | Fact → representations → updates/invalidation → possible disagreement |
| [Behavior preservation](../audit-behavior-preservation/SKILL.md) | Old/new paths → triggering inputs → observable differences |
| [Enum/state shape](../audit-enum-shapes/SKILL.md) | Flags/tags/payloads → transitions → valid variants |
| [Derivability](../audit-derivability/SKILL.md) | Manual impl → semantics/bounds/errors/wire → equivalent derive or retained implementation |
| [Type granularity](../audit-type-granularity/SKILL.md) | Distinction → actual uses/mix-ups prevented → keep/refine/merge |
| [Raw strings](../audit-magic-strings/SKILL.md) | Occurrence → role/owner → admission → uses → text boundary |
| [Raw primitives](../audit-magic-numbers/SKILL.md) | Occurrence → unit/frame/policy → owner/admission → uses → native boundary |

Return the pinned revision, deduplicated findings, compact source-backed traces,
per-lens coverage and limits. Link the ledger for detail.
