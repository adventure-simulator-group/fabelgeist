# Independent review of capture readiness q2303

Reader: `/root/verifier_scope`. This is a new, bounded source assessment, not
an adopted queue disposition or an additional finding. The prepared report by
`/root/mhr_character` was registered by its exact SHA-256 before display; its
observed publication HEAD does not establish that the report belonged to that
commit. Current source was read at
`95e63628f9d70b8dd6e6fede64334696c6995009`. Source windows, receipts,
unread complements and exact bytes are in this packet. File-level identity
checks separately compared the two historically reviewed source files with
`e2755a732c84e67296b5e3441a4259a84f461b2f`; no historical product-source body
was personally redisplayed for credit.

## Findings and qualifications

1. The prepared report's narrow gate is supported. For a frame reaching
   `ready_for_frame`, positive warmup is decremented and returns false. Only
   when `scenario_frame == 0` and `required_motion()` is present does the
   processed-motion/presentation branch reset on a failed predicate and
   require two successful readiness-function calls before returning true.
   The driver sets `applied` after that return. Independent earlier checks
   can return for missing playback or, except FullRagdoll, a bind pose.
   Sources: `capture/readiness.rs:6-30`, `capture.rs:213-238,450-464`, and
   `capture/admission.rs:118-124`.

2. The two successes are not shown to be two rendered or visually settled
   frames. They need not even be consecutive driver invocations: an earlier
   playback or pose return bypasses `ready_for_frame` and its reset. That
   qualification concerns source control flow, not evidence that such an
   interruption actually occurred. `capture.rs:606-617` contains a different
   `settled` counter, which is reached after `applied` and uses retained
   thresholds of at least 60 for the first frame and two for later frames.
   The scheduling source registers the driver in PreUpdate and the harness and
   capture chain in Last; actual engine execution or rendered output was not
   observed.

3. `motion_is_processed` asks whether any key in `clips` has the requested
   motion ID (`animation/mod.rs:254-261`). `presentation_is_settled` currently
   calls `authored_pose_is_ready`, which tests that the plan is not bind
   (`animation/playback_state.rs:84-91`). Those predicates alone do not prove
   the selected playback is the required motion on the intended rig or that a
   visual result has settled. The subject loop stores the last observed
   playback, while missing or unready subjects can return early; supported
   subject cardinality and identity remain owner questions.

4. The `applied` gate is a frame-acceptance condition, not a no-mutation
   guarantee. Before the readiness call, the shown driver can update the
   simulation tick and procedural clock, transition posture, project
   translation and facing, advance a capture action, and project motion
   (`capture.rs:363-448`). Other early returns or rejection paths can
   intervene. Whether those pre-acceptance changes are intended and how they
   affect the final image remain policy and runtime questions.

5. The separate committed ledger record
   `REVIEW-CAPTURE-PREAPPLIED-READINESS-TIMEOUT-001` already owns a conditional
   candidate. `capture_frame` returns for `!applied` before it can call its
   local `wait_or_fail` counter (`capture.rs:559-604,900-913`). The scheduled
   harness is separate: no harness starts ready and returns from its update;
   with a harness, its counter increments for unready equipment only when
   preparation is not active, and explicit failure has its own path
   (`harness.rs:204-220,263-329`; `capture_schedule.rs:39-50`). The fixed
   local limit is 1,200 (`animation_viewer/mod.rs:70`). This establishes a
   conditional gap in that named local counter, not that all watchdogs are
   absent, assets actually fail, or the whole program hangs.

## Recovered and reconstructed relationships

- Original 059 queue entry 2303 has the exact named disposition
  `rejected-suspicion-source-qualified`, with discovery closed only within
  that recorded scope and no runtime pass or global closure. Its selected
  display is a reconstructed subset derived from the original queue stream,
  not a copy of that 21.6 MB stream. Its original transport commit, blob and
  SHA-256 are recorded in `HISTORY-REGISTRATION.json`.
- The historical parallel `lens11_reconcile` e275 read receipts attribute
  their own capture and readiness source reads; the proposal says root
  adoption pending. Separate historical root adoption records list PL11-1092
  and PL11-1094 as adopted without a root reread. Both wordings and their
  chronology remain intact. The parent packet's nine captured historical
  files were independently hashed against its stated values, but their older
  private-capture chain was not re-proven here. All nine also match the
  published transport inventory by SHA-256, byte count and relative-path
  suffix; inventory matching alone is not independent original extraction.
  The selected displays are reconstructed excerpts of those captured records.
- Original 059 queue entry 1214 is an open Lens 12 readiness, native-failure
  and timeout question with no assessed source identities in that row. It is
  related to, but is not established as an exact alias of, the separately
  attributed current partial ledger candidate. The earlier committed prior
  assessment and current ledger have their own commit identity and attribution.
- Equal Git blobs and SHA-256 for historical e275 and current 95e
  `capture.rs` and `capture/readiness.rs` establish only those file bytes,
  not a whole-tree identity, transferred personal-read credit, or historical
  scheduling equivalence. No checkpoint 3821 queue or ledger is reconstructed
  or implied.

## Open questions and limits

Confirm valid subject multiplicity and required-motion/rig identity; the
intended meaning of settled presentation and whether successful readiness
calls must be contiguous engine or rendered frames; allowed state changes
before acceptance; asset-failure, startup and external watchdog paths; and
whether q1214 and other records are exact aliases or related obligations.
The prepared report's narrow source reasoning is corroborated with the
additional interruption and distinct post-application counter qualifications
above. Neither historical nor current discovery, any audit lens, or the full
queue is closed by this packet.

Evidence-only checks can verify checksums, selected-record reconstruction,
Git object identity and the registered display scope. Source was inspected
as text only; product code, fixtures, shaders, macros, builds, tests, viewers,
rendering and runtime probes were not executed. No implementation or
product-validation authorization is implied.
