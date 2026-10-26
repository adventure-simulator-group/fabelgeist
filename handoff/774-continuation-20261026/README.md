# Fabelgeist continuation handoff to segovb

Resume the unfinished #774 audit and the connected #780/#826 implementation
from this checkpoint. The CI recovery is merged. A separate GLB export PR is
ready for review, and the next texture-view change is preserved as WIP.
Neither original implementation PR nor the full audit is complete.

This is a handoff branch, not a product change to merge into main. The user
requested a Git branch handoff rather than a ZIP. Product changes are on the
separate branches below. Work was performed in the main physical worktree;
continue there rather than in the old source copy included with the intake.

## Read first

1. Read `CURRENT.json`, then `AUDIT-RESUME.md` and `SOURCE-LOCATORS.json`.
2. `evidence/` mirrors the ignored `target/recovery-20261025/` directory.
   Restore it into a clean ignored directory in the main worktree before
   switching back to an implementation branch. Preserve any existing evidence
   rather than overwriting a newer checkpoint. Original paths in receipts are
   provenance; use the projection in `SOURCE-LOCATORS.json`.
3. Read the current repository `AGENTS.md`, `crates/AGENTS.md` and applicable
   owning guides before source edits. The original recovery branches contain
   the focused audit skills and ownership/receiving records. Fetch them when
   needed; do not download or read the whole frozen source tree.
4. `MANIFEST.json` records every transported file's exact bytes and hash, plus
   omissions. This directory disables Git text conversion so historical source
   and receipt hashes survive Windows/Linux transfer.

`CURRENT.json` and this README supersede historical status/current aliases in
the raw records. Keep historical receipts as evidence; do not mistake an old
"pending" field, partial source read or narrow review for final acceptance.

## Published implementation

| Work | Branch or PR | Exact revision and status |
| --- | --- | --- |
| CI recovery | [#986](https://github.com/adventure-simulator-group/fabelgeist/pull/986) | Merged as `4530fc19ce16641df65eae28735d69e86bbaa8ae`; current main at handoff. |
| Checked GLB framing | [#987](https://github.com/adventure-simulator-group/fabelgeist/pull/987), `codex/checked-glb-framing-review` | `2ef288d1085c249fa056156182ea626a060bb8d1`; ready for review, open, unmerged. |
| Typed texture-view errors | `codex/826-view-error-wip-handoff-20261026` | `2e8b0cb0728bbcde65bb91c6e04c5488aac6fa23`; WIP based on older main `990430b77a9f63548ffcb642074185f61ddb3322`. |

#987 checks actual serialized JSON/BIN lengths, padding and total frame size
before the final fallible reservation. Typed errors preserve serialization,
layout, allocation and filesystem causes. An unencodable optional timing
diagnostic is dropped after the wrapped operation. Two growth ceilings were
lowered. Geometry, texture generation and equipment assembly are unchanged.
Independent source, rules and final wording reviews are complete for this scope.

The [final full hosted run](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/40339807509)
passed on that exact head: Linux and Windows succeeded, all 48 required contract
groups had positive pass counts and no failures, 11 named export cases passed,
and the 10 required Windows cases passed. The opt-in fixture exporter was
correctly skipped. Evidence is in `evidence/glb-main-4530/`, particularly
`linux-evidence-002.json`, `windows-evidence-002.json`,
`final-hosted-run-002.json` and `published-ready-001.json`. The final published
PR body matches its independent wording review after line-ending normalization.

Do not broaden those successful-path checks into oversized allocation,
reservation failure, 32-bit or filesystem-failure execution claims. Earlier
serialization/image allocations retain their behavior; PNGs may already have
been published and filesystem output is not transactional.

## Next implementation

Keep #987 as the next review/merge checkpoint. The user reviews and merges each
individual PR; agents must stop and escalate when each PR is ready. Do not
merge automatically or build a deep stack.

After the user merges #987, start the texture-view slice from fresh main. Read
`evidence/gpu-main-4530/preflight/integration-plan-001.json` and its path
manifest. Recover only the saved commit's delta; do not merge either old
#780/#826 recovery branch wholesale. Fifteen GPU paths were unchanged/absent
between the saved base and `4530fc19`; the workflow was the only overlapping
path. This is a preflight prediction, not an applied or tested integration.
Recheck the new main and dependency context first.

Preserve every #986 workflow gate, exporter guard, cache/timeout adjustment,
positive-execution check and Windows portability step. Insert the saved texture
validation steps into the current workflow rather than replacing it with the
old file. The canonical validation-scope implementation and typed error causes
must survive later #780/#826 composition. Historical reviews do not clear a
newly integrated revision; complete fresh affected reviews and full hosted CI.

Broader #780 generated/native/core/physical/material/export/caller and lock
composition, the rest of #826, and #774's remaining discovery, repairs and final
acceptance are still unfinished. Follow the saved ownership transfers and
receiving records at each connected function.

## Working constraints

- Use existing tests only. Add no tests, cases, test helpers, assertions or
  doctests; adapt existing fixtures only when required by the change.
- Run project builds, tests, lints, scanners, generators and runtime/DB checks
  on hosted infrastructure. Do not rerun them locally. Git/source inspection
  and evidence/hash bookkeeping are allowed.
- Use one implementation owner for reproduce/fix/test loops. Diagnose the
  earliest failed contract from bounded output. Parallel reviewers may own
  separate source-only audit scopes; they must save exact frontiers.
- Keep tactical tick state in transient Bevy, remove compatibility paths within
  scope, and regenerate schema bindings through the required workflow.
- Wiki prose needs a separate branch from main and Bruno's exact-wording
  approval. No wiki work was performed in this continuation.
- Preserve unrelated untracked `crates/fabelgeist-compute/src/reduce/compiled.rs`,
  `scratchpad.rs` and the old `fabelgeist-audit-774-handoff-20261025/` source copy.
  They were not changed or transported as implementation work.

The evidence bundle deliberately excludes large duplicate CI diffs, generated
scene/image artifacts, and the old source clone. Retrieval references are
provided. No ZIP, new worktree, merge, database operation or public conversation
snapshot is part of this handoff.
