# Deferred finish: pre-repair Rule Review and ownership accounting

R1 remains actionable at frozen `5a34b608ccacc1950313b501d30b036f0a4f8b5b`.
This report is discovery input for the next repair gate, not post-repair
validation, a complete thirteen-lens verdict or final clearance. The proposed
single context submission owner fits the actual failure boundary. Completing
the caller/error/state graph still requires #780 propagation and exact retained
#774 handoffs; no source was changed by this review.

The agreed API is:

```rust
WgpuContext::submit(&self, CommandEncoder)
    -> std::result::Result<SubmissionIndex, wgpu::Error>
```

Root owns this GPU context boundary, fallible KernelBatch::submit with canonical
native-cause mapping and callbacks only after Ok, and standalone
ComputePass::dispatch consuming it. There is no proposed ComputePass::submit
helper, second unchecked API, compatibility shim or alternate copy backend.
The latest parent assignment establishes that design agreement; archived older
ownership metadata is not a competing current API authorization.

## Exact source and evidence

Pinned main remains `990430b77a9f63548ffcb642074185f61ddb3322`. Source remained
clean and frozen at the HEAD above. All 126
`deferred_submission_pre_repair.files` identities match. The source inventory's
115 locations were individually compared with current file lines: 113 submit
occurrences and two direct record locations, across 67 files. Of those, 90
invoke KernelBatch submission, 23 are raw provider submissions and two are
direct ComputePass::record recipients. The raw count includes the batch owner
and standalone owner; these categories are not 115 independent architectures.

The companion `deferred-finish-rule-review.json` records the inventory hash,
67 source hashes, all four exact archived claim-file hashes, every source
location/signature/context, matching claims with exact JSON pointers, and
path-only neighboring claims kept distinct from whole-function matches.
Embedded WGSL and the nested Reduce helper were distinguished from their real
enclosing Rust functions. It is accounting evidence, not a project scanner or
executed acceptance result.

Read all four `774-records/records/parallel-ownership/*claims.json` structures
for relevant exact scope/transfer entries; unrelated claim bodies are not
reinterpreted as GPU ownership. Root hosted-formatting path entries authorize
formatting hunks, not semantic repairs to whole functions. Exact archived
function claims do not themselves establish implementation or validation;
those files explicitly leave family status to the execution ledger.

No source edits, builds, project checks, tests, cases, test helpers, assertions,
doctests, runtime, push or hosted CI were performed. This review wrote only the
two requested ignored accounting reports. The root's new all-thirteen discovery
gate remains incomplete; this report does not open it.

## Failure boundary and proposed owner contract

The independently verified SDK 29 chain is preserved in the earlier
`ranged-copy-rule-review.md` erratum. Copy/pass commands are queued during
recording and validated when encoder.finish encodes them. Current recorder
scopes have already popped. KernelBatch::submit (`kernel/mod.rs:354-359`) and
standalone ComputePass::dispatch (`compute/pass/mod.rs:28-31`) finish/submit
unscoped. A logically valid same-buffer zero-byte copy returns Ok from recording
but is rejected at finish before native zero-size skipping. Default uncaptured
handling can panic; a returning handler can leave an apparent index/Ok and run
callbacks after rejection. No typed source wrapper can recover a cause emitted
outside its scope.

The single helper must own two actual provider phases under the existing
effective native observation policy: finish and observe its failure before
queue submission; then submit and observe that failure before returning Ok.
The canonical GPU return type can directly retain the actual wgpu::Error and
its source. Higher-layer wrappers map that concrete cause once to their owning
typed error. A new generic formatted error, all-purpose submission error or
duplicate provider identity would weaken this contract.

Ok means the operation was accepted under the selected observation policy. It
does not prove GPU completion, readback completion or hardware-wide execution
correctness. Nonblocking borrowed contexts and Wasm remain explicitly
device-handled; they cannot acquire a synchronous verdict by changing the
return signature. Do not falsely document every Ok as an observed validation
success on those configurations.

KernelBatch must gate callbacks on successful helper return. RecordedDispatchCount
still counts recorded dispatches, not validation/execution success; its existing
meaning need not change. Readback::record_pending (`readback.rs:127-139`) attaches
mapping in a callback. A failed submit must not invoke that success continuation
or later treat mapping readiness as evidence that rejected commands ran. When
the batch is consumed on failure, pending callback/state lifetimes need an
explicit disposition; silently leaving a separately held PendingReadback
forever Pending is not a completed lifecycle contract.

## Exact archived ownership intersections

All pointers below are zero-based JSON pointers in
`774-records/records/parallel-ownership/rig_garments-claims.json`. Repeated
submissions in one function retain one whole-function owner.

| Current function or fixture | Exact archived claim and required disposition |
| --- | --- |
| Buffer::read | `/claims/273` owns concrete native/Wasm readback causes; the new helper's caller conversion must be reconciled with this retained function. |
| ArmorGpu::build_in, nearest_points | `/claims/270` includes both complete functions plus device_error and creation/read helpers. Their submission propagation must close the typed GenerateError cause family together. |
| metal::bake and textures | `/claims/362` explicitly owns textures/bake and connected record functions. `/claims/361` owns MetalGpu methods and device_error. Existing String output cannot be independently patched by another agent. |
| DeviceBreastplate::record | `/claims/71` owns the entire method and rig-boundary relationships; both line 117 and 138 submission stages need the same owner handoff. |
| Fit::run, shells, record_skin in device_underlayer.rs | `/claims/40` and `/claims/296` own these whole functions and retained layer/skin contracts. All five submission sites must consume the final API coherently. |
| DeviceWearer::read_frame_async | `/claims/289` owns the complete frame failure/readback graph, including FrameFailure. |
| generate_bracer_on_device_async | `/claims/153` owns both base and morph submission stages. |
| generate_breastplate_on_device_async | `/claims/154` owns the torso generation/morph consumer. |
| GpuGroomer::grow | `/claims/266` owns the whole grow method, including seed/point/guide and exact RNG/native output requirements. A submission-only edit would still touch that claimed whole function. |
| collar fixtures | `/claims/133` names both `intermediate_neck_bulge_and_shared_bib_seam_clear_the_measured_body` and `bib_seating_rechecks_the_shifted_collar_against_a_side_neck_bulge`. All three submissions are inside those exact fixtures. |
| bib fixture | `/claims/132` names `bib_seats_on_lower_equipment_in_the_canonical_wearer_frame`. |
| tasset fixture | `/claims/65` names `wrapped_tassets_reject_nonpositive_final_section_radii` and the other existing wrapped-tasset fixtures. |
| KernelBatch::dispatch / copy and ComputePass::record | `/claims/272` and `/claims/359` are the historical lower-core claims. The later explicit human/core ownership transfer and present root receiver agreement supersede competing lower-core repair ownership; their existence must still be recorded. |
| WgpuContext constructors | `/claims/274` owns new/new_compute/open original cause contracts. It does not name the new submit method or submitted_work_done. A new focused submission owner does not authorize altering those retained constructor bodies. |

The archive has path-neighboring claims in device_fit.rs for fit_recipe,
fit_recipe_async, record_recipe, assemble_recipe and Realization::upload, but
the current direct submit is in **fit_piece_async** at line 173. Do not label it
an exact name match merely because it shares a file. Its call from the claimed
recipe closure (lines 209 onward) makes it a connected fitting dependency that
#780 must reconcile with retained #774 ownership before integrating a split
repair. Likewise an exact constructor claim does not claim every context
method, and a formatting receipt does not transfer a fixture's semantics.

Other inventoried caller functions have no exact claim in these four archives.
Their #780 caller-propagation authorization comes from the current agreement,
not an inference that a whole GPU crate was unowned. The JSON preserves every
such location, including adjacent claimed-path caveats. No conclusion here
authorizes a second independent repair inside retained #774 whole functions.

## Error propagation and retained-state traps

Most production batch callers already return anyhow::Result or HairGpuError's
owning Result and can propagate a fallible submit through existing canonical
conversions. That observation is not permission to suppress the outcome with
`let _ =`, `.ok()`, a logged warning or a fabricated default index.

Material distinct paths requiring complete propagation are:

- ArmorGpu::device_error (`gpu/mod.rs:359-360`) currently accepts Display and
  stores Arc<str> in GenerateError::Gpu (`error.rs:32`). Mapping the new native
  error through that function preserves text and erases its source. Merely
  adding a submit map_err does not complete the retained #774/F19 cause graph.
- MetalGpu methods and metal::bake return String and format device errors
  (`gpu/metal/mod.rs:53-127`, bake.rs:31-97). Their claimed error-owner and
  downstream material/engraving receivers must change coherently. Do not
  preserve String as a compatibility API or create a parallel backend error.
- HairGpuError::Backend already owns anyhow::Error (`hair/gpu/error.rs:44-45`);
  it can retain canonical batch/native errors under supported trait bounds.
  The leaf trait configuration must be proved, not replaced with display text.
- StereoDepth::run (`stereo/pipeline.rs:513-517`) flips history and increments
  frames after submission. Failure must short-circuit those success updates.
- Hair resimulate restores old head/time/history after line 250; step changes
  time/h_last/other CPU state while recording and updates head after line 657.
  LOD generation may cache atlas state after line 240. Propagating Err alone
  does not automatically roll back mutations already made during recording.
  Inventory and preserve the existing mutation order, then define an explicit
  failure/retry policy; do not promise atomic rollback without implementing it.
- Underlayer pushes realizations after lines 223-227; fit_piece_async pushes
  recordings after line 173; breastplate/anime stages read earlier GPU results
  before recording later stages. A failed predecessor must not advance or read
  unproduced data. XPBD/shell interleaved loops must stop after failed submission
  rather than scheduling later substeps/contact projection.
- PendingReadback and read_words/staging paths must preserve associated range
  metadata, buffer lifetime, callback ordering and actual terminal failure.

The only currently infallible production batch API in the direct call set is
KernelBatch::submit itself. Infallible contexts among the broader raw-provider
inventory are submitted_work_done, renderer bind_dynamics/clear_dynamics and
Bevy scalp upload. Void test fixtures and existing test helpers are separate
from production API guarantees and need controlled operand/error adaptation.

## Native, Wasm and external-provider constraints

- Correction to the initial discovery receipt: real per-thread error-scope
  identity requires wgpu's Cargo `feature = "std"`. The native `cfg(std)` alias
  in build.rs does not itself enable that Cargo feature. In the pinned SDK,
  `backend/wgpu_core/thread_id.rs:7-29` uses std::thread::ThreadId only under
  `cfg(feature = "std")`; otherwise every thread receives the same unit key.
  Direct GPU/compute/math dependency declarations at frozen 5a34 omitted std.
  The pre-repair plan therefore explicitly includes native GPU wgpu `std` and
  its manifest/lock accounting. An unrelated feature-unifying dependency must
  not be relied on to establish isolated GPU-library behavior.
  ErrorScopeGuard is neither Send nor Sync; finish/pop and submit/pop must stay
  on the same synchronous thread, with real thread identity enabled on native.
  Do not retain a guard across callbacks, await points or a batch lifetime.
- The exact helper return type preserves wgpu::Error without requiring a new
  Send/Sync wrapper. `wgpu/src/api/device.rs:774-825` makes ErrorSource and Error
  Send/Sync conditional on SDK cfg(send_sync). `wgpu/build.rs:7-10` enables that
  cfg on native and fragile non-atomic Wasm. The GPU manifest enables the
  fragile-send-sync-non-atomic-wasm feature, but browser atomics are not proved.
  Existing unsafe Send/Sync on WgpuContext does not establish Error trait bounds
  and must not be used to justify new unsafe error implementations.
- Native pollster/wait code must remain excluded from Wasm; that dependency is
  native-only. A Validation scope captures that error class, not automatically
  Internal/OOM/device-loss outcomes. Match documentation to the actual selected
  filters and policy; do not claim all possible provider failure is captured.
- The helper receives a native CommandEncoder. Its originating device is a
  provenance precondition: a foreign encoder's finish errors go to its own
  device sink, not a scope pushed on the supplied context's different device.
  Batch and standalone pass currently create encoders from their own context.
  Preserve that actual lineage at every migrated direct caller; the new
  signature alone cannot prove arbitrary external encoder membership.
- The marker in submitted_work_done submits no encoder and owns a distinct
  queue-completion protocol. Do not manufacture an empty encoder simply to
  mechanically route it through the new method or change the marker semantics.
- HairRenderer stores actual device/queue ports, while Bevy scalp upload uses
  RenderDevice/RenderQueue. They do not own WgpuContext. Do not synthesize fake
  adapter/instance/context state or create a second helper to pretend these are
  context-backed operations. Their current provider ownership, error/state
  contract and inclusion/exclusion disposition require explicit accounting.
  In particular scalp upload removes queued items before raw submit, and
  bind_dynamics/clear_dynamics commit retained renderer state after it.
- SrgbConverter::convert already scopes finish and submission separately
  (`srgb_converter.rs:157-175`). Consuming the shared helper must retain its
  construction/recording scopes and failure precedence; do not regress its
  currently correct actual-boundary observation while unifying submission.

## Fixture and acceptance constraints

Existing fixture inputs, values, iteration counts, assertion count/order and
production scheduling must remain. Adapt existing fallible returns/operands to
the final submission API; do not add a test case, fixture, helper, assertion or
doctest. Void test functions, Vec/tuple-returning existing fixture helpers and
thread closures cannot receive a blind `?`. They need an explicit existing-test
error path or a coherent existing signature adaptation by their owner. Production
unwrap/expect remains forbidden; test-only existing unwrap policy is not an
authorization to add assertion coverage or hide a failed production result.

Root must finish the source-backed thirteen-lens pre-repair inventory before
implementation. #780 must supply complete caller/error/state/fixture propagation
and retained #774 handoffs. Required hosted checks and documented baseline
reductions remain future acceptance work, along with full final same-pushed-SHA
audit waves and all three independent reviews. None of this discovery grants
post-repair or whole-PR clearance.
