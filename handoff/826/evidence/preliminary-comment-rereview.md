# Preliminary Comment Review: correction re-review

Status: **independent corrections accepted; shared dependency unresolved**.
This is a targeted re-review, not final Comment Review or audit clearance.

## Snapshot

- HEAD: `71a2d1ffc6203c251f87a4c4e759670058d620b3`.
- Main: `3a50b708381ad00c116445c2c999a49c3943088b`.
- Working-source identity: the 14 file SHA-256 entries in
  `target/pr826-audit/post-comment-snapshot.json` supplied with this dispatch.
- Earlier complete preliminary scope and findings:
  `target/pr826-audit/preliminary-comment-review.md`.
- Applied the previously read Comment Review skill and repository rules.

## Dispositions

**C1, independent files: corrected.** Read the revised `blocking_validation`
documentation, `submitted_work_done` explanation, borrowed-context test rationale
and diagnostic text in `globals/wgpu_context/mod.rs`, and the revised 2D
constructor rationale in `texture_2d/mod.rs`. They now describe the project's
nonblocking borrowed-device policy without claiming that later submissions
perpetually extend `wait_indefinitely` or that owned devices finish instantly.
The latest-at-poll-entry description agrees with pinned
`wgpu-types-29.0.4/src/lib.rs:229-264`. The explicit submission-index explanation
agrees with the method's `queue.submit([])` and `PollType::Wait` descriptor.
Browser completion remains described as callback based. The test change alters
only its comment and failure diagnostic; the assertion predicate and surrounding
inputs/oracle remain unchanged in the reviewed diff.

**C1, claimed shared function: unresolved dependency D774.**
`compute/pipeline/mod.rs:45-47` still says draining validation readback is only
safe when nothing else submits. It remains untouched pending explicit #774
ownership agreement. The corrected context documentation does not establish
that restriction as a wgpu guarantee. Reconcile this wording with the agreed
shared pipeline policy before final clearance; this is not an accepted deferral.

**C2: corrected.** The original readback documentation is removed from
`Texture2d::write` and attached to `Texture2d::read_image` in
`texture_2d/image.rs:97-104`. Reading it with the method confirms the placement:
the method reads raw bytes, reorders BGRA, narrows supported float formats and
expands supported single-channel formats. The move adds no executable behavior.

**C3: corrected for the current producer set.** The `view_error.rs` module and
enum introduction now cover resource admission and native whole-texture view
errors. `Uninitialized` describes required absent native texture state;
`MissingView` describes required absent cached view state. Neither variant now
implies that its producer necessarily performed a format-selection request.
Native-variant request/cause documentation remains accurate. These descriptions
fit current upload, blit, conversion and distance-field producers as well as
`view_with_format`. Revalidate if the pending canonical error/state contract
changes those producers or variants.

**Guide wrapping: accepted.** Read the directly affected paragraphs in
`texture-views.md`. The wrapping changes preserve the stated historical evidence
limits and final hosted-check requirement; they introduce no new validation
claim.

## Limits

No additional substantive finding in these correction hunks. Review was limited
to the requested corrections and their directly affected documentation. No
source edits, tests, project checks, builds, runtime probes or external posts
were performed. The report itself is the only file authored in this turn.

F2/F3 cache/state repairs, D774 ownership/canonical-contract resolution, the
remaining pipeline rationale and final source-dependent guide wording still
require the complete independent review on the exact final pushed revision.
