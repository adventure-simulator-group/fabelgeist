# PR #774 audit handover for segovb

Start with HANDOVER.md, then STATE.json. The initial audits are unfinished.
No implementation or new project tests were added during this audit.

## Contents

- HANDOVER.md: scope, decisions, unfinished work and exact resume points.
- STATE.json: saved initial-lens progress and evidence pointers.
- audit/: complete saved findings, supplements, qualifications, coverage,
  family/ownership ledgers and exported lens instructions.
- EVIDENCE-MANIFEST.json: audit-record hashes and frozen source fingerprints.
- remote-observation.json: last verified PR merge and main metadata.
- PACKAGE-MANIFEST.json: original/export hashes and package exclusions.

This is the audit package, not a worktree or a complete source archive.
It includes seven small recovered Rust source excerpts and two historical
deleted-asset images used as audit evidence, not a repository checkout.

No primary-working-tree directory, GPU patch, unrelated source backups, .git,
build output, dependencies or frozen source trees are included.

## Paths and recovery

Machine-specific paths have been normalized in copies only:

| Placeholder | Meaning |
| --- | --- |
| audit | Extracted audit/ directory in this ZIP |
| HANDOVER_PACKAGE | Directory containing this README |
| SNAPSHOTS | Exact preserved source snapshots from the original handoff |
| REPOSITORY | Recipient's repository checkout |
| IMPLEMENTATION_CHECKOUT | Separate isolated implementation checkout |
| CARGO_HOME / LOCAL_USER | Original external dependency/local paths |

Internal audit-relative paths retain their hierarchy under audit/. Some old
records use continuation/... relative to audit and some use records/...;
resolve them accordingly. Do not run old bookkeeping scripts until their
placeholder paths are configured. They are not project validation tools.

Recover actual code from the original handoff's preserved Git refs or supplied
snapshots. Do not substitute current main for a frozen source variant just
because filenames match. The pr-working snapshot includes four earlier
preserved integration/quality edits; account for that overlay. Verify exact
source versions against the unchanged source fingerprints.

## Hash interpretation

Path normalization changes exported metadata bytes. Historical SHA-256 fields
inside records still identify original bytes. PACKAGE-MANIFEST.json maps each
original SHA-256 to its exported SHA-256. Actual source fingerprint values have
not changed. Neither matching bytes nor a manifest proves semantic clearance.

Preserve corrections and qualifications; do not sum local findings as unique
global defects. Native REG056 duplicates an existing source role, not a new
variant or additional producer/owner delta.

The remaining initial audits, P16/P17 integration, connected repairs, final
audit waves, independent reviews and same-head hosted validation remain due.
Do not merge further PRs or mark #726 complete.
