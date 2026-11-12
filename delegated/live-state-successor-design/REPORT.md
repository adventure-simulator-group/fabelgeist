# Advancing live audit state while preserving historical checks

A separate live-state namespace is a viable design to investigate. It is not
an established equivalent verifier, an implementation, an approval, or a
claim that a lens or queue item is complete. Preserve a completed published
checkpoint C as the historical input, run its unchanged checker against an
exact C checkout, and verify evolving state and evidence separately. Choose C
after the in-flight checkpoint is published; this inspection is of committed
publication revision a9b8331ae347542b1ea707b1b04eb8092735d1bd, not staged work.

At a9b the pinned source authority is 95e63628f9d70b8dd6e6fede64334696c6995009.
All 13 lenses and 3,713 indexed 059 queue records are still incomplete; the
1,031 ledger entries are entries and relationships, not independent defect
counts. The original 3821 queue and ledger remain missing. The 059 records,
reconstructions, historical summaries and current-source work keep their
separate revisions and attribution.

## Why four frozen files are not enough

- The a9b driver selects 698 tracked root verifier names and 54 native evidence
  verifiers when no extra matching files exist. It discovers root `verify_*.py`
  and `native-evidence/*/verify.py`, and substitutes qualified privacy
  successors. New matching files silently change what that old driver runs.
- Its qualified `verify_delegated_links_13_059_public.py:20–29` scans root
  `DELEGATED-EVIDENCE-CHECKPOINT*.json` and builds a map whose later matching
  keys overwrite earlier values. A new root checkpoint can alter historical
  matching even if the four state files are unchanged. Several other checks
  compare exact `rglob` or `iterdir` file sets in old packet directories.
- `verify_main_advance_evidence.py:141–156` requires the frozen 13 lens states,
  transition-cell statuses and predecessor queue counts, so even justified
  intermediate transition coverage can conflict. Particle, cloth and action
  verifiers bind particular old partial queue rows, role states, assessments,
  old ledger records and zero counters. Data-directed paths reach registries,
  tree inventories, archives, source proofs and additional verifiers.
- At a9b literal searches find 375 root verifiers naming the 95e baseline, 413
  progress, 273 059 reconciliation, and 448 role coverage. At least 249 match
  one zero-lens pattern and 312 one zero-queue pattern; these regex counts can
  miss chained or indirect assertions and must not be treated as exhaustive.

A9b has 23,314 tracked entries and 3,071,869,772 summed blob bytes across
paths, not a measurement of disk usage or an independent copy. The retained
NUL-delimited Git tree listing identifies paths, modes, objects and sizes; it
is not a full content rehash or a complete dynamic dependency map. Public
recovery and local object availability need explicit verification.

## Concrete candidate design

1. Identify C, its full tree and qualified historical driver, source records,
   privacy successors and discovery set. Preserve the entire old tree and all
   data-driven dependencies, not merely baseline, progress, reconciliation and
   coverage JSON. Reproduce C in a separate writable scratch checkout, compare
   expected paths, modes and unfiltered blob bytes before and after the old
   evidence-only driver, and record actual inputs, results and optional checks.
   The recovered verifier creates a temporary native Git clone under its own
   root; P17 verification creates a separate temporary object-only repository.
   A read-only mount alone therefore will not replay the unchanged driver.
2. Keep C's baseline, queue reconciliation, role coverage and progress bytes as
   historical observations. Put the new authority and all new evidence under a
   disjoint namespace such as `live/`: a baseline/revision record, per-row
   reconciliation and source/lens coverage, connected repair ledger, append-only
   dated changes or corrections, source receipts and a separate verifier. The
   exact schema is undecided. Never let a new root `verify_*.py`, native verifier,
   historical checkpoint glob, or old packet directory silently join the old
   checker's discovery set. Treat corrections and reopenings as explicit events,
   not as an assumption that every field changes monotonically.
3. Derive live counters from individually identified rows and evidence, rather
   than accepting hand-edited totals. Bind each original 059 source pointer and
   identity to its recorded revision; bind status changes to named evidence,
   outstanding scope and reviewer attribution. Preserve every old ledger
   relationship and its origin, while explicitly recording new, amended,
   reclassified or superseded work. A complete lens requires source inventory,
   role/queue coverage, source delta, cross-lens relationships and unresolved
   questions to be reconciled under an independently reviewed closure rule.
4. Independently compare the whole live tree against C: classify and bind every
   added, changed, removed and mode-changed path, including any landing page or
   privacy reconstruction, and bind every inherited evidence reference to the
   exact C blob or a separately identified qualified successor. Check new live
   assessments, registered displays, source proof chains, manifest members and
   report/ledger links against actual live bytes. Do not accept a scratch overlay
   check as proof that the live target files match it.
5. Give readers a clearly authoritative live landing and retain the old
   landing as a time-bound observation. At a9b `CONTINUATION.md:3–14` calls the
   old files the current authority; leaving that prose unqualified after a real
   closure would mislead. A filename search found six indirect text references.
   The selected prescan verifier only checks its prior receipt, but other
   dynamic/archive uses were not exhausted. Any changed root landing must be a
   bound successor while old checks replay the C version.
6. Carry a fresh, dated main observation in live state without relabeling 95e
   or silently adopting a newer revision. If authority advances, reconcile the
   full source delta and distinguish its new assessment from earlier evidence.
   Publish the historical identity, live manifest, verification outcomes and
   recoverable references at every checkpoint; record same-machine custody and
   optional private/Git checks honestly.

## Privacy and open questions

At a9b the 15-file path-digest reconstruction proposal remains unapplied;
522 observed corpus matches and 273 additional semantically selected fields
have distinct evidentiary bases. Its public verifier does not prove live
integration, private replay or complete privacy. If the tip is corrected,
retain original and reconstructed identities with a reviewed crosswalk and
replay the old exact C checkout separately. A forward change does not erase old
Git history or prior copies. Freezing all old files in the new tip would also
freeze those known values, so privacy correction and immutable history need
separate treatment; neither can be silently traded away.

An independently reviewed live schema and closure rule, a complete dynamic
input and discovery audit, byte-exact historical replay, live-target binding,
privacy review and recovery verification remain open. This packet implements
none of them. It does not establish whole-goal completion, product behavior or
independent-machine custody.
