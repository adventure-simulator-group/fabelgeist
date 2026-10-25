# Initial exhaustive discovery

This is the user's new acceptance target for existing Goal #726 / PR #774.
All twenty migration families remain mandatory. P16 and P17, including shared
producers and consumers, must finish and integrate before final clearance.
All thirteen bespoke-audit lenses, independent Code Review, Rule Review,
Comment Review, and required hosted CI must apply to one final pushed revision.
No overall Goal completion or PR merge is authorized.

Read the current `bespoke-audit` and assigned focused skill. Run one lens at a
time. Parallel readers partition that same lens; overlapping evidence does not
substitute for another lens. The first active lens is
`audit-producer-propagation`. No further source repair, commit, integration,
push, formatter, baseline change or CI request is allowed until exhaustive
initial discovery across every lens is complete and shared repairs reconcile.

`audit-ledger.json` is authoritative for audit/review coverage. The parent
`execution-ledger.json` remains authoritative for family implementation status.
Evidence shards are inputs, never independent clearance. All discovery starts
incomplete. Search hits, historical verdicts and green CI are not coverage.

## Frozen inputs and scope

`source-manifest.json` pins 946 path variants across 833 distinct paths:

- `pr-working`: current PR source at 6533e444fa plus four already existing
  uncommitted equipment-owner edits, compared with main 3a50b708381.
- `rig-quality`: preserved complete cbbbca08 delta against 6533e444fa.
- `report-p17`: preserved complete report/capture delta90a422a against18360adb;
  its shared motor dependency is still unfinished.
- `physical-p16`: every staged/unstaged/untracked source delta in the frozen
  camera worktree at a1203909. This implementation is unfinished and mandatory.

The archive preserves exact deltas and changed-file contents, including
untracked source. Treat overlays as planned deltas, not whole branch restores.
Record both variants and the coherent reconciliation when their functions
conflict. Do not accept the old unmodified branch bodies as the final union.
The original stash remains preserved; do not restore it wholesale or import
unfinished scanner changes. Its recovery handoffs are provenance, not clearance.

Read every assigned scoped file, including existing tests, configuration,
macros, embedded code, deleted baseline paths and generated-source owners.
Follow affected unchanged producers, trait implementations, stored fields,
callbacks, callers and actual receiving bodies to their real endpoints. Add
these paths to evidence with snapshot, SHA and source line/function references.
No file prefix, fixture, private helper, comment or catalogue is a blanket
exemption. Group only identical proven reasons. Read applicable root/scoped
AGENTS files with normal precedence and the user's supplied root instructions.
User style is imports, modules, constants/statics/type aliases, structs, enums,
traits, impls, public/private functions, then cfg(test) module.

## Evidence requirements

Each owner writes only its own ignored evidence shards below
`wave-01/audit-producer-propagation/OWNER/`. Do not edit the shared master or
another owner's files. Keep a progress file with exact first unfinished source
cell and partial coverage; never silently mark an unread path complete.

Account for each `source_id` in `assigned-OWNER.json`. A coverage row must point
to concrete contract/trace records, or give a source-backed non-applicability
reason. Record observed source SHA. If only part of a file is read, record its
ranges and leave it partial. Deleted files require baseline-to-owner trace.
The source manifest plus frozen archive allows missing/deleted baseline reads.

For each distinct producer contract, record its meaning, canonical owner,
admission, first producer, retained state, operations, every materially distinct
receiving path and actual wire/framework/numerical/text endpoint. Inspect all
primitive escapes and accessors from the focused skill, including intermediate
locals and unchanged receiving bodies. Link identical paths explicitly rather
than replacing proof with a checklist.

For each candidate give disposition: actionable defect, migration gap,
intentional proven boundary, non-applicable, strengthening suggestion, or
unresolved/blocker. Give exact location and scenario, governing rule if relevant,
smallest connected repair, shared functions/owners/dependencies and concrete
acceptance criterion. Prior pending work may address a finding, but inspect it
and map its exact source; do not infer that it already integrates or passes.
No new deferrals are authorized. Do not narrow an unfinished family into future
work. Send concrete shared-function issues to the coordinator for deduplication.

Use JSON/JSONL plus concise Markdown source traces if useful. Record progress
incrementally; the inventory is large and must survive context resets. Finish
the current assigned lens shard, then await the coordinator's next lens.

## Execution limits

This phase is read-only for all source and Git state. Do not spawn agents or
post/send externally. Source/Git/rg and stdlib text/JSON/TOML/hash/ZIP metadata
are allowed. NO Cargo/build/check/test/fmt/Clippy/metadata, Rust parsers,
scanners/generators, numerical/runtime probes, or local project checks. Use
existing tests as source evidence, add none: no cases/assertions/helpers or
doctests. No source or runtime PASS is inferred from static metadata.
