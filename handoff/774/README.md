# Continue the #774 bespoke-type audit

This package transfers Bruno's and Adler's unfinished audit of Fabelgeist's
bespoke-type migration. It is self-contained evidence for a developer who has
not read Bruno's chat. Start with this file, then `CURRENT-STATE.json` and the
four investigator handoffs it references. This document supersedes older
handoff timing, task assignments and progress summaries inside the evidence.
The older records remain source evidence; their timestamps and qualifications
must be preserved.

Repository: https://github.com/adventure-simulator-group/fabelgeist
PR: https://github.com/adventure-simulator-group/fabelgeist/pull/774
Overall migration: https://github.com/adventure-simulator-group/fabelgeist/issues/726

## Immediate task and acceptance

Finish the remaining initial discovery for all thirteen bespoke-audit lenses
before writing the execution plan or repairing source. **Start from Adler's
completed admission (03) and type-ownership (06) audits with their findings and
evidence. Do not require a fresh exhaustive baseline reread.** Reconcile only
later material and changed context for those completed snapshots.

The end goal is one final pushed revision where all thirteen lenses, Code
Review, Rule Review, Comment Review and required hosted CI pass together.
#774 is already merged; follow-up repairs require their own branch/PR. Do not
merge further PRs or mark overall #726 complete. Historical green CI is not
validation of later source. There have been **zero completed full audit waves
and zero cleared lenses** in this audit continuation. Discovery completed with
findings is useful evidence, but is not repaired-source clearance.

After every selected initial lens has a complete inventory, reconcile defects
by remedy and shared function, write the implementation plan, and execute it.
Then repeat complete audit waves and affected reviews until their evidence
applies to the same final revision. Do not silently narrow scope, lose old
findings or defer unresolved source/policy questions. Search results and
conversion counts are navigation signals, not semantic clearance.

Use `skills/bespoke-audit/SKILL.md` and each of its thirteen focused skills.
They are the actual captured review lenses, not a paraphrased checklist.
`skills/rule-review` and `skills/comment-review` are included as well. Read the
repository's applicable AGENTS files before subsequent source work.

## Revision authorities

`preservation/source-pins.json` gives full hashes and the bundle's prerequisite.
The main frozen audit sources are:

- Merged #774: `13fc9dcc71ab879de364117e5462310673e43210`.
- Frozen pre-merge source catalog: `6533e444fa37552927070cc8d8190907fd3b413e`.
- Comparison base: `3a50b708381ad00c116445c2c999a49c3943088b`.
- Preserved P16: `039329518e264f19206cf5b0b98d177433f40c17`.
- Preserved P17: `90a422a453cd990ebc7f0ecf2ad5ed3e2d57d696`.
- Preserved rig work: `cbbbca08c89a8768926799371f54cf6057312e5e`.

The actual Bruno checkout remained at `58ea2ca24b34e9b10960a0cd67d978384d659c3b`.
**That is an older ancestor, not a later main snapshot.** Do not diagnose its
older code as a post-merge regression or replace the frozen catalog with it.
The four dirty Cargo manifest lint edits are preserved separately in
`preservation/unrelated-four-lint-manifests.patch`; do not apply them to this
migration or treat old #774 CI as their validation. Original stash
`6c109560050d97dd6deeb842b1d71ec8a280949b` is included in the Git bundle without
being popped, applied or rewritten.

When moving implementation to current main, inspect actual new commits and
revalidate affected contracts. This handoff does not assert current GitHub
main or any other agent's current head has been freshly checked today. The
frozen merged delta has 38 paths and is a separate audit dimension.

## Authoritative records and exact frontiers

All paths here are relative to this package. The audit ledger is
`audit/ledger.json`. Its 173 local records overlap with Adler's 1,002 adopted
records in `audit/reconcile-adler-20261023/adler-finding-records.json`; these
numbers must not be added as a globally unique defect count. Findings contain
source-backed traces, dispositions, repair/dependency requirements and
acceptance criteria. Follow their linked packets for exact evidence.

Continuation directory (abbreviated `C` below):
`audit/reconcile-adler-20261023/audit-continuation/`.
Original source-role catalog:
`audit/reconcile-adler-20261023/recovered-sources/catalog.json`.
Shared additional source material: `C/source-extensions.json`, **173 entries**,
with bounded ranges and hashes. A registration is not audit coverage.

| Lens | Preserved discovery status / next work |
| --- | --- |
| 01 Producers | Baseline and material 0–61 retained with findings; later material 62–172 needs its own bounded reconciliation. |
| 02 Existing owners | Baseline and material 0–61 retained with findings; later material 62–172 pending. |
| 03 Admission | Adler baseline, 246 overlays, 38 merged paths and material 0–99 accounted with findings; only later 100–172 pending. |
| 04 Operations | 2,853 roles, 246 overlays, 38 merged paths, material 0–103 accounted with findings; later 104–172 pending. See `C/operations/checkpoint.json`, closure 678. |
| 05 Aggregates | 1,981/2,853 original roles; next 1697, strategic_scene/visibility.rs line 1. Overlays/merged/material dimensions remain incomplete. See `C/aggregate-states/FINAL-HANDOFF.json`. |
| 06 Ownership | Adler baseline, 246 overlays, 38 merged paths and material 0–99 accounted with findings; later 100–172 pending. |
| 07 Redundant authority | Baseline, overlays/merged and material 0–99 accounted with findings; later 100–172 pending. |
| 08 Behavior | 1,469/2,853 roles; next 1441 with exact partial source/diff ranges saved. See `C/behavior-preservation/FINAL-HANDOFF.json`. |
| 09 Enum shapes | 2,853 roles, 246 overlays, 38 merged paths and material 0–159 accounted with findings; later 160–172 pending. See `C/enum-shapes/STATE.json`, closure 734. |
| 10 Derivability | 1,473/2,853 roles; next 1464, camera/config.rs line 1. See `C/derivability/FINAL-HANDOFF.json`. |
| 11 Granularity | 385/2,853 roles; next 385, physical-p16 combat/consequence.rs line 1. See `C/type-granularity/FINAL-HANDOFF.json`. |
| 12 Raw strings | Unfinished. Load the actual focused skill and inventory; other-lens review does not substitute. |
| 13 Raw primitives | Unfinished. Load the actual focused skill and inventory; other-lens review does not substitute. |

Exact owner checkpoints and saved partial ranges win over dated navigation
summaries. Some original roles are partial ranges, not entire files. Historical
role counts retain explicit schema/navigation qualifications and duplicate
corrections; never count fresh supporting rereads as new roles. No completion
is inferred for unread tails or truncated output.

Root's latest aggregate packets are 139–156. New reconciled findings include
F010 (leaf triangle metadata counts a different population from the emitted
mesh), F011 (one visible vista ring can sample an unrendered second ring), and
F012 (aliased place selections overwrite the building's original relocation
pose before moving furniture). F009's terminal medical-state admission trace
is retained. Each packet distinguishes source reasoning from runtime evidence.
Do not turn a conditional API counterexample into an observed gameplay bug.
One apparent frozen fixture type mismatch in strategic_scene/views.rs was
already fixed in merged #774; it is explicitly qualified, not a current defect.

## Mandatory preserved implementation and coordination

The twenty-family propagation plan still applies. Do not drop P16/P17 because
#774 merged or replace implementation obligations with audit gates. Relevant
records are:

- `historical-family-execution/execution-ledger.json` and its
  `parallel-ownership/` claim files: original authoritative family/claim record,
  now historical relative to the merged audit, with closures and validation
  evidence beside it.
- `audit/reconcile-adler-20261023/family-history.json` and
  `preserved-increment-blob-map.json`: source versus integrated/merged blob
  comparisons for P16/P17/rig, not acceptance verdicts.
- `audit/reconcile-adler-20261023/ownership-takeover-evidence.json`,
  `assignments/`, `partitions/`, and `audit/coordination/`: exact prior claims,
  exclusions and successor handoffs.

Carry the earlier six accepted families and their evidence forward. Shared
repairs may satisfy several family requirements but do not automatically
complete those families. Before source changes, claim whole affected
files/functions in one ignored map. The first trace owner repairs a shared
function for every affected family; coordinate or explicitly transfer it,
never add competing types or temporary primitive adapters. Keep one coordinator
for contracts, integration and final validation, one owner per fix/check loop.

#780 owns related fitting/body/hair producer contracts; #826 owns sorting and
GPU layout/dispatch contracts, including reconciliation with #779. #895 owns
its archive/NumPy boundary work. Existing coordination evidence includes #826's
partial successor repairs and exact sparse-group/compute-entry qualifications.
Consume final canonical owners and recheck their actual status before touching
shared paths; do not duplicate their types, claim their whole scopes or assume
an unintegrated successor commit has fixed merged main. Bruno-machine chat IDs
are historical metadata, not portable task identities. External messages need
applicable human authorization; this package has not posted anything.

## Constraints and known evidence limits

- Add no tests, cases, assertions or fixture helpers. Use existing tests only.
- No local project checks/builds/scanners/generators/browser/GPU/DB runs were
  authorized during initial discovery. Continue that audit-only gate. Later
  implementation uses hosted CI after a source-complete connected family or
  repair batch; do not push to trigger CI while known paths remain unfinished.
- Follow producer → retained state → operations → unchanged callers/receivers
  → actual boundary. `.into()`, `.encoded()`, `::from`, casts and extracted
  `x.seconds()` arguments are canaries, not automatic defects or exemptions.
- Represent semantic absence explicitly. No infinity meaning “cannot arrive,”
  no -1 meaning initial state, no fabricated physical support at zero. Keep
  valid local numerical reduction identities and final native padding distinct
  from retained semantic sentinel states.
- Reuse existing bespoke owners; preserve precision, signed/zero roles and
  coordinate frames. No compatibility shims, legacy fallbacks or blanket
  wrappers. Prefer “bespoke type” in contributor-facing prose.
- Tactical tick state stays in transient Bevy, not SpacetimeDB. Generated DB
  bindings are generated from schema, never hand-edited. Do not reset a public
  or player-bearing database.
- Keep original stash, unrelated source and all old evidence. Audit reports
  stay ignored. No merges; no overall goal-complete claim.

Current evidence does not establish final CI, all-lens or Code/Rule/Comment
Review clearance. Native/renderer acceptance has explicit limits: external
Goslar/12-input terrain and color manifests, calibrated renderer verdicts,
older creator SIGSEGV/control evidence, Detail/Facade image differences and
unfinished material tests must be evaluated from their dated source and owning
handoffs. Do not call an inherited baseline failure proven for a newer head
without matched control evidence. Q023–Q025 and other exact open source-policy
questions remain in investigator handoffs. Installed standard-library source
corroboration is labeled separately from frozen-lock dependency proof.

## Portable use

`audit/` contains the complete audit directory, recovered source objects,
original Adler audit exports, GitHub discussion snapshots and current ignored
records. `adler-original/` contains every extracted member of his supplied archive.
The original ZIP container is omitted to keep each Git blob below GitHub's
file-size limit; original member bytes remain preserved.
`skills/` contains all thirteen lenses and coordinating/review instructions.
Original absolute paths are evidence and have not been rewritten or rehashed.
Use `portable_paths.py` to resolve them locally; use source ID plus SHA-256,
never filename alone, to distinguish PR, preserved and merged variants.

To verify this transfer without running project checks:

```sh
python3 verify_package.py
python3 portable_paths.py source 1697 1 160
```

For implementation, clone the public repository normally, then import preserved
objects without applying them:

```sh
git clone https://github.com/adventure-simulator-group/fabelgeist.git
git -C fabelgeist fetch origin 3a50b708381ad00c116445c2c999a49c3943088b
git -C fabelgeist bundle verify /path/to/package/preservation/source-revisions.bundle
git -C fabelgeist fetch /path/to/package/preservation/source-revisions.bundle 'refs/handoff/*:refs/handoff/*'
```

The compact bundle requires public base 3a50b708 and includes every named
source pin, original stash and isolated P16/P17/rig work. Do not checkout/apply
the stash wholesale, overwrite a newer main or auto-apply the unrelated patch.
The frozen source evidence is usable without importing Git objects. If a future
old/new comparison needs another historical source, fetch the exact recorded
commit and verify its expected hash before crediting it.

This is the unencrypted branch edition. The branch `MANIFEST.json` covers
these exact published files; `PUBLICATION.json` records the original ZIP hash
and this publication's packaging changes. No password is required.

Current exported helpers in `C/handoff-tooling/` contain original-machine
absolute paths; treat them as historical tooling and adapt their root explicitly
before reuse. `portable_paths.py` is the ready portable reader. No task is
running in the background for Adler to wait on.
