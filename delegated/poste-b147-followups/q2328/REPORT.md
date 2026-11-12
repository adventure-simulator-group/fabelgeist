# Bounded q2328 source reconciliation

Historical 059 entry `/entries/2328`,
`REVIEW-L11-CREATOR-DRAPE-CANCEL-REQUEST-TAG-ASSOCIATION`, carries the
`rejected-suspicion-source-qualified` disposition. Its named discovery was
closed only within the old dispatcher scope; it did not close its coarse parent,
other lenses, the global alias search, repairs, or runtime validation. The
original 059 queue and its path-redacted reconstruction have different full
hashes. Their exact q2328 byte slices match. This is earlier recovered evidence,
not the missing checkpoint 3821 final ledger or queue.

The original reviewer's four whole-file reads at
`e2755a732c84e67296b5e3441a4259a84f461b2f` belong to
`parallel lens11_reconcile`; the original receipts recorded root adoption as
pending. Later `segovb` records adopt the delegated bundles at that same
revision without claiming a root reread, current-target reconciliation, other
lens credit, validation, or a count change. The 13 referenced records are
recovered by their recorded hashes. All four named e275 files are byte-identical
to the corresponding pinned 95e files, and each of the eleven captured old or
current source paths has a commit-to-tree-to-blob proof. This is a per-file
comparison, not whole-tree, dependency, asset, or runtime equivalence.

## Current bounded source reasoning

All following paths are under `crates/adventuresim-character-creator/src/` at
`95e63628f9d70b8dd6e6fede64334696c6995009`.

- `drape_worker.rs:11-20,53-59,71-127` holds one active handle, a single
  replacement vector of `(tag, input)` pairs, its active tags, cancellation
  flag and preview slot. `request` cancels the old request and clears its
  latest preview. While cancelled, `poll` does not return its preview or
  finished outcome. After the old handle is joined, the pending vector is
  unzipped once into tags and inputs in the same order and the new thread gets
  fresh cancellation and preview objects. Even if an old callback races with
  the clear, it has the old objects; the cancelled progress is suppressed and
  the replacement has a different slot. This supports the historical rejection
  for serial requests and polls through this dispatcher. It is source reasoning,
  not an execution or a proof for changes that never issue a replacement.
- `outfit.rs:44-72` forms and reverses whole `(InventoryItemId, DrapeInput)`
  pairs; `studio_generation.rs:32-72` submits them when preparation succeeds
  and its separate change policy selects a drape. `drape_preview.rs:22-27,
  165-225` clears ready state on a request and zips returned garments with the
  active tags; it can retain usable garments alongside warnings or an error.
  `garment/outfit.rs:26-81` processes the inputs in order, makes previews
  from a completed prefix plus the current garment, and returns immediately
  on a failed garment. Thus this inspected native producer returns a prefix,
  not a later garment shifted into a failed one's position. The second consumer
  `wardrobe_tab.rs:68-76,114-120,143-147,229-267` requests one `()`-tagged
  garment and takes its first preview. These are selected producers and
  consumers, not an exhaustive caller or invalidation inventory.
- A distinct checkpoint restart question remains. `drape_worker.rs:61-64`
  clears stored checkpoints for a requested fresh simulation, but if a handle
  is still active, `poll` then takes that handle's outcome checkpoints even
  when cancelled (`:91-104`) and moves them into the pending request
  (`:110-121`). `garment/outfit.rs:37-55` passes previous per-index checkpoints
  to draping and records checkpoints from an attempted outcome. If such an old
  outcome contains reusable stages and no later check invalidates them, the
  new request can receive stages after the clear. Whether an affected request
  is reachable, which stages are present or eligible, and actual reuse or
  visible impact are unestablished. This is a different first operation from
  tag association and from the saved-binding and body-cache questions. Do not
  assign an alias or additional unique count on this evidence.
- `drape_worker.rs:36-39` only sets the cancellation flag when dropped; the
  method itself does not join. Pending work waits for an active handle to
  finish (`:86-127`). Bounded completion and cancellation latency are open;
  no scheduling or actual liveness failure is claimed.

The q2323 reviewer `/root/verifier_scope` separately traced private creator
body/model fitting and corrective cache identity; q2324 reviewer
`/root/mhr_character` separately traced hypothetical probe/fit policy. Their
sealed report summaries are copied for relationship review, without transfer of
personal source-read credit. q2325's native skin/layout review and q2326's
hair worker/body Arc cache identity review are separately attributed. The
current ledger snapshot selected `REVIEW-CREATOR-CORRECTIVE-CACHE-POLICY-001`
and `REVIEW-SAVED-DRAPE-DIRECT-WEAR-UNVALIDATED-BINDING-001` as related
context; neither establishes the restart question. The literal `checkpoint`
search returned 508 matches but displayed only the first 30, and searches are
bounded metadata navigation, not an exhaustive alias search.

Open work includes the native lower-level stage eligibility rules and real
checkpoint contents, all restart/request call sequences, cancellation latency,
producer invalidation and the actual supported inputs. Preserve independent
owner attribution and ask central reconciliation before assigning any alias,
count or repair order. No product code, assets, shaders, renderer, fixture,
build, test, probe or repair was executed.

## Read and recovery limits

`READ-LIMITS.json` and 14 numbered source receipts record the exact current
ranges this reviewer personally displayed. Whole-file capture, Git proof,
comparisons, retained historical pointers, and other reviewers' work do not
convey additional current reading credit. The historical pointer capture was
saved in full but only the registered selected metadata and bounded root views
were displayed. The broad root metadata and a later directory locator display
were tool-truncated; their full captures, subsequent bounded views, an initially
unregistered equality/filename locator and failed connector-capture attempts
are recorded in `READ-QUALIFICATIONS.json`. Physical-worktree searches are
unpinned navigation only. Hashes prove byte identities and object links, not
comprehension, independent custody, source semantics or runtime behavior.

The verifier requires separately preserved raw current and historical commit
objects, both full queue anchors, and the time-bound ledger snapshot. Those
anchors include material unsuitable for indiscriminate publication and are not
part of the scan-cleared packet. Source and historical evidence here remains
qualified by the bounded scanner limitations in `PRIVACY-QUALIFICATION.md`.
