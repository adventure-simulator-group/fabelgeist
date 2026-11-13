# Independent Lens 3 review: saved schedule admission

Reviewer: `final_guard_peer`. This is an independent, bounded peer review of
`wrapped_scope`'s q2545 assessment at product revision
`95e63628f9d70b8dd6e6fede64334696c6995009`, tree
`1eec97de340d9b7debbb30ca0c0fb2240a73b93f`. The evidence checkpoint
`8caeb2003d10d08320566206817aa0c694231778` supplies the queue and ledger
snapshot. The historical q2545 Lens 3 obligation is pending, and its 8ca
wrapper remains unassessed. This review proposes no adoption, closure or
unique-defect count. It is later than the sealed 36-root process cutoff.

The selected source supports the author's conditional admission observations:

- `OrganizationAllocation::Scheduled` publicly exposes an `ActivityMinutes`
  and `String`. The generic minute type accepts zero, so a directly constructed
  `Scheduled` value can pair zero with an empty string and its getters return
  zero and `Some(empty)`. The parser rejects any organization paired with zero
  and rejects a blank positive ID; it trims an accepted positive ID
  (`crates/adventuresim-core/src/strategic_schedule/allocation.rs:7–75`). A
  directly constructed positive allocation can also retain padding that the
  parser would trim. These are representable API states, not proof of a
  supported direct caller, committed invalid row or required identity policy.
  Zero minutes with no organization remains a valid `None` state.
- The selected web form validates its allocation, and the native update reducer
  validates the same numeric fields and organization pairs, reconstructs the
  canonical saved row and checks membership/practice permission before update
  (`crates/strategic-web/src/schedule.rs:11–39`,
  `crates/strategic-web/src/routes/settlements/party/training_activity.rs:18–35,70–97`,
  `crates/adventuresim-stdb-module/src/time/stationary.rs:195–262`). The core
  validator takes the activity array and organization-minute pairs separately;
  the selected callers pass matching values, but arbitrary other callers and
  aggregate policy remain open (`allocation.rs:104–131`).
- The isolated simulation reducer requires an owned run and a fresh ordinary
  solo character, among other checks, but tests the schedule's total against
  the day limit before assigning the raw allocation to the saved row. Later
  fallible work follows (`crates/adventuresim-stdb-module/src/simulation.rs:
  253–263,349–493`). A one-minute reading allocation with all other fields
  zero is below the daily total yet fails the 15-minute numeric admission
  (`allocation.rs:28–35,157–195`; `crates/adventuresim-core/src/activity.rs:28`).
  If all other guards and later transaction conditions succeed, that value is
  committed, and the selected non-camp stationary or `Some(location)` preview
  path reaches the numeric validator, the shown `expect` encounters the error
  (`crates/adventuresim-stdb-module/src/time/stationary.rs:68–85`,
  `crates/adventuresim-stdb-module/src/time/schedule.rs:71–89`, and
  `crates/strategic-web/src/templates/settlement/character_skills.rs:
  2103–2112`). The simulation scope, actual admission, commit or rollback,
  preview location and runtime outcome are not established here.

`ValidatedSchedule` checks the numeric day budget and segment sizes without
organization IDs or persistence policy. The native stationary path filters
current organization eligibility on a temporary clone and may zero an
activity while retaining an ID before numeric validation
(`time/schedule.rs:155–192`; `time/stationary.rs:68–85`). A zero-minute plan
with a retained nonblank ID is therefore numerically acceptable while the full
pair parser rejects that saved pair. The web preview's `None` location returns
a clone without the numeric check; its `Some` branch uses the numeric check.
This distinct temporary/numeric contract is not itself a defect. Also, the web
adapter normalizes a blank optional ID to `None` before full parsing; direct
native parsing sees the raw pair. Their error and acceptance difference for a
zero/blank pair is a policy question, not an observed product discrepancy.

The default allocation is zero with no organization IDs. The selected NPC
initializer writes the initial plan only for an adult, not-yet-initialized,
unallocated schedule (`npc_causal.rs:195–250`). Its shown producer uses 360
labor minutes and 60 plus 15 times a bounded index for socializing. If the
index honors its declared half-open contract, total time is 420–480 minutes
in 15-minute steps (`crates/adventuresim-core/src/npc_policy.rs:123–135`,
`crates/fabelgeist-determinism/src/stream.rs:48–60`). The external sampler and
an NPC run were not executed. Any future repair decision should preserve valid
empty/default and NPC modes as well as the distinction between saved-pair and
temporary numeric schedules; producer inventory, public-construction policy,
transaction behavior and the correct owner remain open.

## Historical and ledger relationships

The original q2545 selected row records 6533, the 1–99 source requirement and
no own-Lens assessed range. Its separate Root Lens 2 supplier receipt is
navigation, not transferred Lens 3 coverage. A separate recovered receipt
credits Root with an independent 6533 Lens 3 read of the same file. The e275
receipt records no fresh personal e275 reread and makes reuse conditional on
frozen-source adoption, which remains unproved. These are historical
attributions, not this reviewer's or `wrapped_scope`'s new 95e credit.
The three named source revisions have the same single-file bytes; twelve
selected historical q3301 files also match their 95e counterparts per file.
Neither comparison equates entire trees or recovers checkpoint 3821.

The q3301 original row first frames the `Some(location)` preview `expect`;
its separately qualified historical contract revisit identifies the guarded
simulation writer's total-only admission. Both connect to the conditional
current source chain above. Their revision, status and reader remain separate;
this is not a global alias or an additional accepted defect. q628 is an older
Lens 11 zero/blank public-variant question with no linked source receipt and an
unresolved original locator; its reconstructed-ledger journal text matches
that original question. q3444 is a distinct Lens 12 organization identity and
string question. Its separate physical-p16 receipt credits `lens12_frontier`,
and its retained audit object matches the 95e allocation file per file; that
does not transfer its reader or prove all current consumers.

Ledger 89 is an existing partial aggregate question about parallel activity and
organization durations; ledger 1120 is a planning relationship to that work.
Those records, q3462 and the q3301 writer/preview chain keep their distinct
first operations and open reach, policy and alias questions. Historical q2494
is another saved-schedule ownership question. No full alias inventory follows
from the bounded search.

The selected original and reconstructed q2545 bytes match, while the complete
059 queue streams have different hashes. The original checkpoint 3821 queue
and ledger remain missing. Protected extracted evidence was read only and was
not restored into or modified. Full replay depends on the separately held
original queue, historical extraction and Git objects on the same machine;
this is not independent custody or backup.

I personally displayed 17 prospectively registered 95e windows over 14 files.
The first aggregate display was tool-truncated; all windows were then displayed
in six smaller untruncated groups. A display-receipt successor corrects my
initial mistaken count of five groups. Full source blobs were retained only for
identity, and display is reviewer-attested rather than independently telemetered.
The registered route and NPC-policy windows include test-annotated text, read
as source only. My earlier q2561 Lens 7 review had read the adjacent schedule
window including lines 720–727; that overlap and the earlier scope do not
transfer credit. No product code, test, build, fixture, runtime probe, repair or
product validation was run.
