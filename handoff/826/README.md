# PR #826 unpublished implementation handoff

This branch preserves work requested by Bruno for Adler Halbe to resume on
another machine. The implementation is present directly in the normal source
tree, with its original commit ancestry. It is unfinished work, not a request
to merge or a claim of audit or CI clearance.

- Published PR: https://github.com/adventure-simulator-group/fabelgeist/pull/826
- Original public PR head: `d2e9f2f2984d33073386f0267a673fb7ac6478a3`.
- Preserved local committed head: `19a6b62687713b9ecfa7aa137003f11bd662baf9`.
- Source snapshot including 0 uncommitted paths: `19a6b62687713b9ecfa7aa137003f11bd662baf9`.
- Main already integrated in this ancestry: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Companion recovery: `codex/780-recovery-handoff-20261025`.
- Full #774 handoff: https://github.com/adventure-simulator-group/fabelgeist/tree/codex/774-audit-handoff-20261025/handoff/774

## Start here

Fetch this branch and the companion recovery branch. Create a new local branch
from this recovery branch. Read `evidence/ledger.json`, then follow the current
receiving and ownership records before editing shared functions. The ledger
contains historical checkpoints; exact source hashes and revision-qualified
receipts determine what applies. Older `REPORT-current.md` and historical
"current" descriptions do not override this handoff or the latest ledger.

Read ledger.json.resume and its latest material receiving receipts.
Complete #780 ead46619d material receiving audits and shared receiving
repairs, then compose the canonical #780 core/physical/caller/lock graph.
The #826 generated-program, texture/buffer/dispatch/native contracts
must remain canonical. Finish the hosted-acceptance workflow only after
actual existing test targets and final connected signatures are known.

These two branches deliberately preserve their separate histories. Do not
merge them wholesale or overwrite shared files from a historical packet.
Reconcile complete functions against the ownership transfers and newer
receipts. Rebase or integrate current main after checking which contracts
Adler's now-merged #774 work changed; the recorded integration base is not a
claim that these snapshots include today's main.

## Evidence and portability

`evidence/` contains the ignored audit ledger, source-backed traces, plans,
ownership transfers, review reports, source snapshots and comparison evidence.
`referenced-tmp/` preserves available temporary files named in the ledger.
`skills/` contains the thirteen focused audit lenses and coordinating/review
skills. Original absolute paths inside evidence are provenance: map the old
worktree's `target/pr826-audit/` to this `evidence/` directory and the old
source root to your checkout. Companion worktree references use its recovery
branch. Do not rerun historical helper scripts without checking their paths.

`manifest.json` records file hashes, omitted duplicate #774/binary archives,
and any split historical file. Split parts concatenate in listed order to
recover the original bytes. Product source files are not split or encrypted.
The original stash and original worktrees were preserved unchanged.

## Acceptance still required

No project checks or hosted CI were run for this publication. Scoped historical
reviews and source comparisons apply only to their recorded source and scope.
They do not establish whole-PR acceptance. Finish connected producers,
retained state and consumers before pushing to a PR branch or requesting CI.
Then reconcile all thirteen lenses, independent Code/Rule/Comment Reviews and
required hosted checks on one exact final revision. No merge is authorized.

Use existing tests only: add no tests, cases, helpers or assertions. Use hosted
project validation; do not run local project builds/lints/runtime/DB checks.
Historical software-provider compute numerical failures, Creator SIGSEGV,
GPU self-skips and absent calibrated renderer coverage remain limitations to
verify or resolve, never presumed passes on these revisions.

Both original PR heads remain unchanged. This recovery branch is published
without opening a PR or requesting CI because the connected graph is still
incomplete. Continue useful implementation rather than restarting the audit;
retain admission/type-ownership evidence supplied by Adler and reconcile it
against the actual source changes.
