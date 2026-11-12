# Static verifier evolution investigation at 8608c88

The source-only investigation identified historical assertions coupled to
mutable continuation files. It does not establish audit completion, adapt a
checker, replay a prior checkpoint, or verify product behavior. The source
revision recorded by the checkpoint is 95e63628f9d70b8dd6e6fede64334696c6995009.

The original checkpoint 3821 ledger and queue remain missing. The 059 records
are recovered for 059, not promoted into 3821. Reconstructed public evidence,
recovered originals, historical summaries, and newly assessed work must retain
their own identities and qualifications.

## Observed coupling

- `check_published_evidence_privacy.py:10-37` discovers root `verify_*.py`
  scripts dynamically, runs five early names first, skips two optional root
  names, substitutes qualified successors for 20 names, substitutes a city
  successor, and also runs `native-evidence/*/verify.py`. At the specified Git
  commit there are 694 tracked root scripts, 692 selected root names and 54
  selected native evidence scripts. Untracked matching files would also be
  selected by the live filesystem glob.
- `verify_central_particles_13o_95e.py:40-49` binds one specific ledger row,
  asserts two queue rows remain partial, checks the current ledger length, and
  asserts both closure counters remain zero. `verify_central_cloth_13l_95e.py`
  lines 10-13 and `verify_central_collider_13m_95e.py:135-146` similarly bind
  record-specific ledger and partial-queue states. These assertions describe
  historical state; changing the live fields would cause a failure.
- `verify_action_state_95e.py:63-79,155-169` also requires particular role
  statuses and provenance-linked source objects, then checks the partial queue,
  ledger, progress, and latest main observation. `verify_role_worklist.py:10-17,
  50-72,97-130` checks role identity, selected status classes, archive objects,
  current tree data, and evidence links. A three-file progress/queue/ledger
  overlay would not cover role states or their data-driven references.
- A text search over root scripts found 413 naming the progress file, 373 the
  baseline, 273 the 059 reconciliation, and 448 the role coverage file. At
  least 249 match a literal zero-lens pattern, 312 match a literal zero-queue
  pattern, and 288 match a selected historical role-state pattern. The zero
  and status counts are lower bounds: they miss other spellings and chained
  comparisons. Filename matches alone do not prove a particular dependency is
  reached when the driver runs.
- At this commit the checkpoint recorded 13 incomplete lenses, 1,030 ledger
  entries/relationships, 3,713 indexed queue rows of which 2,430 contain the
  exact text `unassessed`, and zero semantically reconciled queue records. The
  2,853 role statuses are recorded in METHOD.json. Other queue prose does not
  itself establish closure, and ledger relationships are not a count of unique
  defects.

## Dependencies and qualifications

The conservative replay input is the complete exact checkpoint tree: 23,030
tracked entries, 3,069,809,657 summed blob bytes across paths, including
preserved archives, manifests, source identities, redacted successors, driver,
verifiers, and their data-driven inputs. That sum is neither compressed object
store usage nor a recommendation to duplicate it. All enumerated modes were
regular files. TREE-LISTING.z.gz preserves the exact NUL-delimited Git tree
listing and METHOD.json records its hash and selected script blob identities.
Enumeration is not a full content rehash or proof of independent custody.

Outside tree contents, replay needs a compatible Python including `tomllib`,
Git, and sufficient writable scratch. The qualified recovered verifier at
`verify_recovered_public_redacted_plus2.py:7131-7175` clones an embedded
native bundle into a temporary directory beneath its own root. The P17 evidence
verifier at `verify_p17_patch_recovery.py:31-38` constructs a temporary bare
Git object store to reproduce a recorded diff. Neither is a product run, but a
read-only replay root will not suffice without changing the verifier. Preserve
fresh scratch and verify the replay tree before and after; do not use the
protected 28ba extraction. Exact portability, all external dependencies and
side effects were not exhaustively traced here.

The default driver does not supply optional `--git-repo` arguments. For
example, `verify_checkpointc629_recovery.py:30-36` and
`verify_adapter_coordinator_review.py:31-42` explicitly skip their optional Git
checks without one. A default pass cannot be described as replay of private
originals, independent custody, a check of all prior Git comparisons, or proof
of absence of private material. Reconstructed bytes remain qualified; earlier
Git history remains separate.

## A minimal design to assess before adoption

1. Freeze the entire historical input tree and original driver/verifiers at a
   named commit. Re-run that exact historical driver in a separately created,
   writable, isolated scratch checkout only after matching every expected tree
   path, mode and exact blob byte identity, and after excluding unexpected
   matching `verify_*.py` or native verifier paths. Check the same facts after
   replay, recording command, environment, output, exit and any exceptions.
   Git extraction/checkout filters or archive attributes can change bytes, so
   verify the materialized bytes; do not merely trust a commit name. A retained
   Git commit alone is not independent custody, and scratch may need gigabytes.
2. Keep that historical result explicitly separate from a new verifier operating
   on live files. The latter must verify full 059 source/index identities and
   source pointers, per-row current disposition and evidence, ledger identity
   and links, role/source coverage, and the 13-lens status. Counts must be
   derived from these records under an explicit definition, not accepted merely
   because a counter was incremented. A partial, related or reconstructed row
   is not a semantic closure. Preserve prior record observations additively,
   including reviewed corrections or reopenings where needed.
3. Bind all unchanged inherited evidence and explicit approved changes between
   the frozen historical tree and the live checkpoint; check later verifiers
   and new source-receipt/ledger dependencies on the live tree. Otherwise a
   historical pass says nothing about an altered live finding. Preserve all
   privacy crosswalk and attribution distinctions rather than substituting a
   scratch overlay without a binding to the actual target bytes.
4. Record per-lens closure only after its authoritative 95e source inventory,
   historical obligations, relevant cross-lens relationships and unresolved
   questions are reconciled with evidence. If main moves, record the observation
   and handle its source delta explicitly. Keep repair and product validation
   `not started`/`not performed` under the user's present instruction.

This is a recommendation, not an implementation or a proof of equivalence. A
full dynamic input map, semantic review of any successor, privacy review,
actual replay, and a separately evidenced live-state verifier remain open.
