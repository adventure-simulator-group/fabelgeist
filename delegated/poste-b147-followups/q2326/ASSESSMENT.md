# Bounded q2326 reconciliation: hair and body instance identity

Reviewer: `/root/mhr_character`. Current source revision:
`95e63628f9d70b8dd6e6fede64334696c6995009`. The historical reviewer
assessed revision `e2755a732c84e67296b5e3441a4259a84f461b2f`; the five
assigned files have identical individual Git blobs and SHA-256 hashes at both
revisions. This does not establish whole-tree identity or a clean working tree.
The historical record is recovered 059 evidence, not checkpoint 3821.

The historical q2326 rejected the *simple equal-count stale-body suspicion* in
its named scope. At 95e, `hair_preview.rs:123–159` matches both groom and
`Arc::ptr_eq` against the body for placed and in-progress hair; `:163–193`
retains the worker's body Arc with the completed hair. `:271–295` obtains a
body from `StudioCache`, and `:297–343` uses the matching placed style for the
shown hair. The source supports the narrow safeguard: a different body Arc
with the same counts does not pass that placed/worker match. It does not prove
that the Arc represents the current model or that a runtime renderer succeeds.
`character_morphs/hair.rs:11–60` describes the placement and whole-joint
mapping; no joint-layout or animation result is validated here.

There is a related *upstream identity-policy question*, already captured by
historical q2327 and current ledger
`REVIEW-CREATOR-CORRECTIVE-CACHE-POLICY-001`: `studio_cache.rs:33–55` keys
the body by LOD, proportions, identity, and expression, rather than the
current model/correctives. `model_controls.rs:25–37` and `main.rs:430–454`
show a selectable same-LOD corrective change and successful model replacement;
`main.rs:327–335` schedules reload before regeneration and hair refresh after
regeneration, while `studio_generation.rs:32–61` asks the cache for the body.
If the unchanged recipe key is present, the cache can return its old body Arc;
the hair guard then accepts old placed hair or an in-progress worker with that
same Arc and groom. `BodyHead::new` combines model faces, skin and names with
generated positions and joints (`hair_preview.rs:78–101`), so a worker started
before the reload may also represent the earlier model. These are source-level
conditional relations, not proof that supported assets produce a different
body, that a particular redraw runs, or that a user sees a mismatch. The old
placed hair can remain consistent with the old cached displayed body. The
question is whether model/corrective identity must be part of the upstream key
or an invariance guarantee is intended. Attribute the cache-first question to
q2327 and the earlier independent q2323 reviewer; do not add a defect count or
merge unrelated identities automatically.

A separate bounded worker-lifecycle question follows from
`hair_preview.rs:131–159`: when a different groom or body is requested while
the old worker is unfinished, the implementation starts a worker and replaces
the stored handle. Dropping a Rust join handle detaches its thread, and the
source comment explicitly says the older worker is left to finish. Only the
currently retained handle is collected (`:163–193`), so the displaced result
is not installed through that handle. If further different requests arrive
before earlier work finishes, more than one background worker can remain live;
actual scheduling, admission frequency, GPU/resource effects and whether a
limit is desired are open. This is not a demonstrated failure or an additional
ledger count. Refresh defers the request while the studio is dirty, joints are
unavailable or a control is being dragged (`:277–290`). Even when an equal
worker is already under way, `BodyHead::new` is evaluated before the request
check, cloning its source data (`:88–99, 134–139, 289–290`); the actual cost and
performance relevance are unknown.

## Historical attribution and boundaries

Recovered 059 q2326 is preserved as exact original byte slice plus a
reconstructed projection, and compared as a selected JSON object with the
later reconstructed public queue. Equality is for that selected object, not
for the entire original and public queues. The historical full-body reviewer
was `parallel lens11_reconcile`; the root `segovb` adopted the separately
recorded receipts. `HISTORY-REGISTRATION.json` preserves the 13 original
receipt, plan, root-adoption, verification and proposal files, and
`HISTORY-SELECTED-{A,B,C}.json` gives personally displayed selected rows.
Root adoption did not become a new root personal read or other-lens credit.
Recovered q2327 is a separate original projection and current ledger entry
47 is a separate projection; both source records and revision identities are
registered in `RELATION-SELECTION-REGISTRATION.json`. The prior q2323 reviewer
`/root/verifier_scope` independently reported the same-LOD cache issue; their
manifest and source-read receipt identities are in `PRIOR-ATTRIBUTION.json`.
That agent's q2325 skin-layout review is distinct. No personal source credit is
transferred between reviewers or revisions.

Our exact current views are listed in `READ-LIMITS.json`: all five named
current file bodies, and only selected `main.rs` and `model_controls.rs`
windows. Capturing the old file bodies did not constitute a personal historical
body read. One attempted model-controls range exceeded EOF and failed before
displaying any source; its correction was registered before successful read.
A later repeat display of already-registered hair source was not separately
captured at display time, so its reconstruction is explicitly labeled as such.
All test-source text seen is text only. No product, test, asset, renderer,
fixture, runtime, build, repair or product validation was run.

## Questions and bounded lens perspectives

- Admission and invariant operations: equality protects the chosen Arc and
  groom; the upstream cached model dependency and supported toggle semantics
  remain the question.
- Aggregate state, redundant authority and ownership: cached body, selected
  model and prepared hair carry related facts, but this review cannot establish
  the intended authority or whether distinct instances can differ materially.
- Producer propagation and behavior preservation: trace above is conditional on
  a cache hit, successful reload and scheduling; no measured behavior is claimed.
- Type granularity, existing type owners and enum shape: no independent finding
  is established in the selected identity trace.
- Magic numbers, magic strings and derivability: no finding is established in
  this bounded trace; no complete source-wide scan is implied.
- Worker lifecycle: whether to bound, cancel or tolerate superseded work and
  repeated head copies is an owner policy/performance question, not a proven
  runtime defect.

Do not mark the queue, lens, thirteen-lens audit, repair, or product validation
complete on the strength of this packet. `VERIFY-EVIDENCE.py` checks evidence
identities and registered displays, not product semantics or global privacy.
