# Bounded 059 q2359: native GPU readiness identities

## Historical record and provenance

This is a new source-only `verifier_scope` review at the pinned authoritative
source revision `95e63628f9d70b8dd6e6fede64334696c6995009` (95e). It is not
historical evidence of a later checkpoint, implementation approval, product
validation or an exhaustive lens audit. Original 059 q2359 comes from
`56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e`; the separately reconstructed
record is from `26e360920b7b2ec32d5c76238623eaf6d6617d7e`. Their full queue
streams have different hashes, while the exact q2359 byte slices match.
Both identities, offsets and original bytes are preserved in this packet.

The original `L2-EXT-GPU-READINESS-NATIVE-IDENTITIES` record has status
`scoped-field-classification-with-current-alias-qualification`. Its granular
contract explicitly says `audit_discovery_closed_within_named_scope: true`,
`existing_canonical_identity: null`, and new accepted unique count zero;
implementation gate remains false. The named scope is the 50-line native
mesh/image identity snapshot and contains predicate. The old review found no
primitive identity conversion or reuse defect in that scope, and expressly
reserved native/runtime/temporal chains, current source and global aliases.
The e275 same-lens adoption had zero fresh personal lines and did not close
the global/native chain; that historical result is not automatically a 95e
review or 3821 recovery. See `Q2359-SUMMARY.json`, `DETAIL-FOCUS.json`, and
the original exact record.

The original frozen receipt has 16 rows: 1,638 fresh own-Lens-2 body lines,
25 root personal bounded reread lines and 215 inherited same-lens selected
lines, with no added unique count. Its first row preserves the original Lens
2 `camera` reader's attestation for dialogue bindings 1–215 and root's
personal reinspection 216–240; that is reader attribution, not a claim about
a camera subsystem. The later e275 receipt adopted 1,878 selected assessed
lines and zero fresh personal lines; its verifier was metadata-only. The
q2359 primary `gpu_readiness.rs` body matches its frozen `6533e444...` hash,
the e275 source and 95e. This verifies byte identity, not reading credit or
all historical dependencies. Nine direct and two nested cited records were
recovered from a previously sealed q2355–q2356 packet, itself captured from
the protected original extraction; each digest matches. Their 11 transport
metadata matches are same-machine evidence, not independent custody.

## Current source assessment

`crates/adventuresim-tactical-client/src/tactical_scene_viewer/gpu_readiness.rs:14–50`
shares one `Arc<Mutex<ResidentAssets>>` resource between the app and render
subapp. It holds separate `HashSet<AssetId<Mesh>>` and
`HashSet<AssetId<Image>>` collections. The selected cleanup observer fills
them from the corresponding native `RenderAssets` iterators. `contains`
checks the passed mesh handle's native ID and every supplied image ID against
that single locked snapshot. The current bounded reading corroborates the
historical identity representation; no primitive conversion is present in
these operations. It is not an external Bevy implementation audit or a claim
that a snapshot proves rendering, future availability or all resources ready.
The lock's `expect` and source schedule were read as text, with no contention,
poisoning, extraction timing or runtime behavior established.

The building review installs the resource and its observer before capture
(`tactical_scene_viewer/building_review/readiness.rs:14–28`). Its selected
`assets_ready` retrieves the standard material then supplies IDs for present
base-color, normal and metallic-roughness textures (`:112–130`); the building,
door and window checks consume this predicate (`:133–155`). The garden review
passes an empty texture iterator for a branch mesh and six explicit image IDs
for leaf materials (`building_review/gardens.rs:78–110`). For an empty
iterator, the image `all` predicate is vacuously true; mesh membership still
applies. These are actual selected input lists, not proof that all material
fields, render assets or supported capture profiles are exhaustively covered.

The furniture path avoids a second installation when the selected building
review has already installed it (`furniture_readiness.rs:13–28`). It checks
selected standard-material texture IDs and the ground material's extension
iterator (`:73–105`); that iterator returns soil height/AO, stone albedo,
stone ARM and traffic mask image IDs (`presentation/vista/streets/material.rs:97–120,143–152`).
The selected building and furniture observers use 120-second waits on their
own readiness (`building_review/readiness.rs:252–285`,
`furniture_readiness.rs:109–134`). Their presence and native IDs do not prove
a successful capture or settle timeout policy. Continue tracing actual
supported resource producers, material requirements, failure states,
snapshot timing, native scheduling, and first-operation alias relationships
before any broader disposition. No invalid reachable asset, missed texture,
stale snapshot or runtime incident is asserted here.

## Related historical records

Original 059 q2303 `REVIEW-L11-CAPTURE-AUTHORED-READINESS-WAIT` recorded a
separate authored capture progression classification involving required
motion, settled presentation, warmup and two ready checks; it has no `status`
key, and its gate is false. q1214 `Q-L12-9668da87ceace16ff7eb` remains
`open-evidence-or-disposition` for a linked harness global timeout, native
asset failure and readiness question. Both selected original/reconstructed
queue slices compare byte-equal, but these records are historical and their
full supporting dependencies were not recovered in this packet. Do not turn
their distinct temporal/readiness questions into an alias for q2359's typed
identity field classification.

Three selected original 059 ledger objects are captured as reserialized
objects, with corresponding reconstructed objects structurally equal; whole
ledger streams differ. `ADLER-L11-TEXTURE-RESIDENCY-FAILURE-STATE` concerns
loss of a per-recipe terminal failure/cause, and
`ADLER-L11-CITY-GPU-PREPARATION-OUTCOME` concerns pending versus rejected GPU
scratch preparation/stage. Both are older frozen-source qualifications with
current/native proof pending. `SEGOVB-L11-CAPTURE-RUN-ARTIFACT-CERTIFICATION`
concerns per-run screenshot persistence; it says wider readiness remains
independent. They are connected evidence to inspect, not established aliases
or new counts. The coordinator owns reconciliation and the repair ledger.

## Evidence and limits

The packet captures six pinned source files, with seven prospectively
registered displays totaling 820 unique personally read source lines and
4,436 captured but unread lines. The precise per-file ranges, hashes and
registration receipts are in `SOURCE-READ-RECEIPT.json`. The full original
source arrays and adoption records were captured; one oversized combined
historical display was truncated, and a later focused display supplies the
named disposition. A broad ledger locator terminal output was also truncated
while all 88 locator entries were retained; a 10-key subset was displayed
separately. A first locator parser failed before displaying source. See
`DISPLAY-QUALIFICATIONS.json`. Fixed-string navigation, recorded local time
ordering, byte identities and metadata are not independent proof of
comprehension, complete coverage or nonexistence of aliases. The separately
sealed scan receipt records the bounded privacy checks and their private-input
and same-machine replay limits.

Test text, attributes and derive syntax were inspected only. No product
code, tests, builds, fixtures, macros, shaders, renderers or probes ran; no
repair was attempted. The verifier checks evidence integrity and optional
source/transport membership, not semantic correctness or independent custody.
