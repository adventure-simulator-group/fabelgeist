# Independent submission successor Code Review

No new owned implementation finding was identified. R1's receiver repair
remains coherent under the precise logical-device/paired-queue precondition.
Required caller, error and retained-state families remain open. This is a
receiver checkpoint, not compile, source-complete, whole-PR or final clearance.

## Exact source and read limits

- HEAD: `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`.
- Reviewed parent: `f2b5ffb1320bc6d3610e8726ac694546ed1a465b`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Git working tree is clean. All 132 affected working paths independently
  match exact committed HEAD bytes; 129 are byte-identical to the reviewed
  parent. The three changed paths are kernel/mod.rs, submission.rs and the
  texture-views guide. All pinned SDK fingerprints also match.
- Main diff: 77 paths. Binary diff SHA-256:
  `5422a1e47b8eb3985f371d67489ad1cd9dbf6455ce6c76d4b33ba935881afa02`.
- Sorted compact JSON changed-file path/hash map SHA-256:
  `39cafdff168cde0388e1e7bc4ea73e9ca0d66e4995dd961d23b4e3fa8d28db7f`.

The companion JSON stores all 132 hashes, stable-path identities, pinned SDK
hashes and current source locations/dispositions for the 104 enclosing
function records covering the prior 115-location census. No uncommitted owner
source is substituted for either checkpoint.

Freshly read the complete three-path delta, entire current submission helper,
Kernel declaration/comment context and the guide's full submission contract.
The other 129 source bodies were not freshly reread in full. Their previously
inspected source proofs are revalidated by exact byte identity and current
connected-contract assessment below. The changed Kernel file deletes exactly
three obsolete comment lines; every statement, lock, constructor, API and
callback body remains identical. This report does not describe hash checks as
new whole-body reading or inherit a prior whole-PR approval.

## Current connected contracts

The helper now explicitly requires an encoder created by the context's device
and a queue belonging to that same logical wgpu::Device. A shared backend or
adapter is expressly insufficient. This accurately scopes the native error
sink on which the implementation depends. It remains a caller precondition;
the public raw context/encoder API does not independently prove arbitrary
external device/queue pairing.

Reassessed against the exact helper and unchanged pinned SDK proof: a native
Validation scope encloses encoder.finish, and its result is read before queue
submission. A separate scope observes queue.submit. Ordinary finish rejection
therefore stops before queuing; destroyed/mapped/device errors that the SDK
reports at submission remain observable through the queue scope. Both return
the actual SDK error. There is no new GPU completion wait or second policy
owner. Native wgpu std still provides thread-specific scope identity; browser
and borrowed paths still provide device handling without a synchronous
validation verdict. Nonmatching error filters and later device health remain
outside this Validation contract.

The unchanged Batch receiver wraps the concrete error in the canonical
ComputePassError::Native and propagates before its registration-order
callback loop. Standalone dispatch and the ten context-backed texture methods
consume the same helper. No earlier unscoped construction/write acquires a
retroactive scope merely through this helper. The correct converter's own
finish/queue scopes remain intact. Removing misleading compilation comments
changes neither COMPILING mutex behavior nor native scope mechanics.

Full affected-graph relationships remain the same: texture 2d/3d/cube cached
and missing states, selected-view effective format versus logical/native
allocation, face/array geometry and unchanged read/write/clear/blit consumers;
shader/reflection/pipeline authority, supported resource/sampling union and
sparse layouts; provided-variant uniform packing, whole-block bounds, former-
panic typed errors, direct-nonstruct zero behavior and native error sources;
source cache identity/success-only publication; folded launches, empty-first
Batch admission, lazy preparation and uniform arena lifetime. The exact source
identity plus revised finalization assessment introduces no new inconsistency
in these contracts. This does not clear known unfinished consumers.

## Required dependencies remain explicit

E1 remains required: armor/metal boundaries currently stringify or return
String, so their canonical owner handoff must preserve the new native cause.
M1 remains required: reused hair/BVH/hash recording metadata needs its agreed
recorded-versus-submitted failure/reuse contract. A changed submit Result does
not itself make those states transactional.

All #780 caller and existing fixture Result adaptations, manual Reduce,
ranged-copy callers, cache/recipe/layout/count/stride/mode/parser/folded
families and retained #774/F19 body/hair/armor/error families remain required.
Current callers that ignore submit Err can still continue into reads,
publication or pending-readback polling. Receiver acceptance is not complete
connected migration or source-compilation evidence.

Raw HairRenderer and Bevy queue ports now have an explicit source-agreed
device-handled policy. They are not observed-native pass APIs and are not
silently certified by the context helper. That policy does not require this
reviewer to duplicate a native submission wrapper. #780 owns the corresponding
API documentation; its final integration remains coordination work.

No local checks, tests, builds, scanners, generators, runtime or project
execution occurred. No source edits, new cases/helpers/assertions/doctests,
push or CI occurred. Only ignored JSON and Markdown receipts were written.
Required integrations must finish before hosted checks; all13 lenses, Code,
Rule and Comment Reviews and required hosted checks must pass on the same
final pushed revision. No final approval is supplied here.
