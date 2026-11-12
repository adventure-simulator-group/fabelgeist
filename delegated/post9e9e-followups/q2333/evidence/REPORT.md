# q2333: tincture liquid and medicinal dose quantum

The recovered 059 record `PL13-255-TINCTURE-LIQUID-DOSE-QUANTUM-POLICY-PENDING`
is an open connected source and policy question, not an accepted runtime
incident. Its later granular refinement is a documented conditional source
question, while both its exact current/alias qualification and its discovery
closure remained open. The original and reconstructed 059 queue streams have
different hashes; their selected q2333 bytes match exactly. This is not the
missing checkpoint 3821 final queue or ledger.

The original Lens 13 reviewer in `parallel-l13-20261029` personally recorded
whole-file reads at frozen `6533e444fa37552927070cc8d8190907fd3b413e`.
`segovb` later adopted that delegated evidence without a root personal reread or
runtime pass. A separate later own-Lens-13 selected herbalism read was at
`e2755a732c84e67296b5e3441a4259a84f461b2f` and was separately adopted; the
record still says current reconciliation and global clearance were incomplete.
The queue's focused reread qualifications and truncated prior catalogue attempt
remain in the exact recovered entry. Twenty directly cited historical records,
two nested prospective registrations and five frozen source objects were
recovered by their recorded hashes. These events do not become new body or
cross-lens credit.

The five frozen source files are byte-identical to their counterparts at pinned
`95e63628f9d70b8dd6e6fede64334696c6995009`; herbalism also matches its e275
selected-current Git path. Fifteen revision/path pairs and the current clock
path are bound through raw commit, tree and blob identities. Seven located
frozen commit-object copies match byte-for-byte; this does not establish seven
independent custodians. Per-file identity does not imply whole-tree, dependency,
asset, execution or policy equivalence.

## Current source assessment

All source references below are at pinned 95e. The listed ranges are selected
source reading, not product or test execution.

- `crates/adventuresim-stdb-module/src/herbalism.rs:275-332,337-478` admits a
  bottle of exactly 150 ml of spirit and one exact ground-poppy lot, subject to
  custody, skill, setup time and lifecycle conditions. It stores the recipe's
  versioned profile and, for a finite successful skill result, potency clamped
  to 0.1–1.0. `:141-174` materializes that potency in a separate medicinal
  component after maturation. `inventory_container.rs:54-63` stores the liquid
  amount as an integer `u64` millilitre field despite its `water_ml` name.
- `herbalism.rs:503-578` accepts a nonzero requested `DoseMilliunits` no
  greater than one standard dose. It treats that number divided by 1000 as a
  fraction of **remaining** potency and liquid. The potency decrement remains
  an `f32`; the liquid decrement rounds that fraction upward to whole ml and
  at least 1 ml. It administers the rounded medicinal dose, forcing zero up to
  the minimum nonzero value, and deletes the container medicine and liquid
  when remaining potency is tiny or volume reaches zero. The request is not
  automatically an absolute quantity of the original bottle's potency.
- `crates/adventuresim-core/src/physiology.rs:54-92` defines 1000 milliunits
  per standard dose, rounds standard-dose `f32` to the nearest milliunit and
  provides the explicit minimum of one. For an assumed finite 0.1-unit initial
  potency and accepted request of one milliunit, the modeled potency decrement
  is approximately 0.0001 standard dose, while the zero-rounded value is
  promoted to a recorded 0.001 standard dose. This illustrates a policy
  relationship; it is not proof of a patient effect or a supported exact skill
  value reaching that initial state.
- The historical 150-minimum-request example is conditional: with an initial
  150 ml, no intervening changes and 150 successful one-milliunit requests,
  every positive integer volume at or below 150 loses one ml and the volume
  reaches zero. Ideal real-number proportional potency would retain about 86%
  after those requests before deletion; actual `f32` rounding and all success
  conditions were not executed. No observed loss, patient outcome or accepted
  defect is claimed.
- The selected browser control in
  `crates/strategic-web/static/inventory-browser.js:921-932,1048-1056` labels
  the action “Take 10% dose” and sends 100, not 1, milliunits. The handler in
  `crates/strategic-web/src/routes/settlements/party/containers.rs:59-70,
  437-461` defaults to 100, accepts 1–1000 from a submitted form and calls the
  native reducer for the actor as patient. The existence of an accepted form
  value does not prove the normal button submits it or that a sequence succeeds.
  The 100-milliunit path also uses whole-ml rounding, with its exact sequence
  and impact unvalidated.
- `crates/adventuresim-stdb-module/src/disease.rs:2080-2100,2222-2264`
  requires the selected relationship and living actors, validates the profile
  as oral, reads patient time, records administration and commits a terminal
  boundary; the shown component helper has no explicit interval wait.
  `time/clock.rs:12-30` refreshes official world time from the reducer
  timestamp. The authored profile mapping in `item_references.rs:12-22` and
  `physiology.rs:576-585,656-660` identifies `poppy_tincture` version 1 as oral.
  These checks leave an actual repeated successful path, terminal effects and
  timing policy unresolved.

The separate 95e food-preparation relationship
`relationship-preparation-view-material-quantum-95e` concerns a residual food
amount offered by a view but rejected by whole-milligram admission. Its first
operation, representation, reviewer and historical q2321/q2318 links differ.
It is related planning context, not an established alias or shared unique count.
The bounded ledger snapshot returned no `tincture` or exact q2333 literal
matches and one `quantum` relationship; this is not a global alias search.

Open questions include the intended physical liquid versus potency quantum,
accepted rounding and explicit discard policy, what supported callers can send,
real patient survival/relationship and clock sequences, material cleanup and
transaction behavior on errors, and other aliases. Do not begin repair or
product validation on this assessment. No product code, build, test, fixture,
macro, parser, renderer or runtime probe was executed.

## Recovery and privacy limits

`followup/READ-LIMITS.json` lists 33 exact prospective current source displays.
Historical files and captured whole blobs do not create additional personal
read credit. Two broad historical metadata displays were truncated; their full
captures and narrower registered views are retained. A scratch-anchor placement
exception and its unknown prior-content qualification are recorded privately.
The source displays that include test-module lines were text inspection only.

This is an exact **private** evidence packet. It retains local locator and
incident paths and is not automatically cleared for public publication. The
bounded scans and external receipt state their limits. Raw commit objects, full
historical queues and the time-bound ledger snapshot are separately held
external anchors and are not cleared by the packet scan. Hashes show byte
identity and Git object links, not independent custody, comprehension, semantic
truth, policy or runtime success.
