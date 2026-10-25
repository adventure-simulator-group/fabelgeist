# Independent Immediate-checkpoint Code Review

No new actionable owned correctness defect was found at this checkpoint. The
S5 change rejects used native Immediate data before layout construction, while
retaining the intended behavior for unused and shader-local globals. Required
#780 and retained #774 families remain incomplete; no whole-PR or final
clearance is granted.

## Exact source and fingerprints

- Reviewed HEAD: `0bc4fd52bc376a22c6f7e73bc7f9ea427f662f17`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Git reported a clean working tree. The full main-to-HEAD scope remains
  73 changed files.
- SHA-256 of `git diff --binary BASE HEAD`:
  `7b4f1d20c58c84ecf9f5fbf84d821bb19133e8497430b15531f980dcaf30955a`.
- SHA-256 of the independently read changed-file path-to-SHA-256 map encoded
  as sorted compact JSON:
  `f073255a865662868fa8f60e287586cfa6e5947801e8c63e8a790fff2c163425`.
- Every `immediate_repair_checkpoint` and `immediate_wave` fingerprint in the
  ledger matched independently, and both name the reviewed SHA.

The complete delta from the independently reviewed reflection checkpoint
`2eb1129b0a8d09289aab9e7cc83bc3499a8a3d9e` contains exactly three paths:
pipeline/error.rs, pipeline/reflection.rs and texture-views.md. All other
reviewed source relationships were revalidated against their unchanged bytes
and current producer/recipient ports, rather than inheriting an earlier pass.
The ledger records all thirteen reflection-wave discovery lenses complete
before S5 repairs, with owner dependencies still unresolved.

## S5 source-backed verdict

`compute/pipeline/reflection.rs:29-48` now determines static usage across the
union of compute entry points before requiring a resource binding. An unused
global still skips reflection. A used Immediate variable returns
`ComputePipelineError::UnsupportedImmediate { name }` before the binding filter
and before any native bind-group layout is created. A used Immediate belonging
only to a later compute entry point is also rejected; the default entry point
does not hide it from the shared layout's admission contract.

The previous early `variable.binding` filter would have skipped Immediate
state because it is not a buffer/texture resource binding. The new ordering
closes that omission without changing the resource-role or sampling analysis.
Used private/workgroup globals still have no resource binding and skip that
resource assembly after the Immediate check; unused Immediate data still skips
at the usage check. Native shader features and validation remain authoritative
before reflection when they independently reject a module.

The pinned Naga 29.0.4 `ir/mod.rs:371-385` explicitly describes Immediate as
data provided through SetImmediates commands rather than a buffer. Both owned
pipeline-layout construction paths still declare `immediate_size: 0`, and the
owned recording graph issues no native immediate-value commands. Typed
rejection is therefore the correct admitted state for this named-parameter
interface; it does not silently accept an unpopulated native value channel.

`ComputePipelineError` retains Clone/Debug, its structured name payload and
the established Display/Error mapping. UnsupportedImmediate has no fabricated
native source; shader/native error variants retain their concrete SDK sources.
`ComputePipeline::prepare` propagates the typed reflection failure before
committing a pipeline/cache instance. The added guide paragraph accurately
states the limitation and its actual recording cause.

## Full-scope and affected-graph revalidation

Revalidated all three whole-texture view selectors and their controlled
constructors/removal operations, generic/face/array views and direct
copy/read/write/clear/render consumers. Same-format cached Arc identity,
MissingView-before-native precedence, detached cached views, alternate native
admission, device checks and storage sRGB normalization are unaffected by S5.
Selected-view effective format/dimension/device remains distinct from native
allocation format. Actual native causes and original requests remain retained.

Re-read the current shader/pipeline preparation, immutable source/device and
cache boundaries, public pipeline-admitted binding assembler and parameter
selection ports. Immediate rejection precedes native layout generation and
therefore reaches both static and fast consumers through their common
pipeline-admission producer. Complete entry-point keys, retained native cache
outcomes/retries and borrowed/browser validation limits remain unchanged.

The full reflection inventory remains coherent: used resources are the union
across compute entries; sampling_set controls float filtering across that same
union; sampler comparison, multisampling, arrayed view dimensions and storage
access roles retain their native variants. Sparse group indices retain their
slots. Used binding arrays, external textures, unsupported handle/address-space
payloads and unsupported uniform representations retain typed rejection.

Direct/flat supported uniforms still retain names, offsets, spans and
UniformValueShape. Both packers check supplied value shape before writing.
Static missing/mismatched values retain zero bytes; the cached path rejects
them and retains its documented narrower matrix support. IntegerWord preserves
From<i32>'s existing Unsigned(value as u32) representation and native bits.
Multiple blocks per group and nested/array/nonrepresented uniform shapes remain
typed admission failures. Checked static allocation/upload, logical buffer
prefixes and canonical fast errors are unaffected by S5.

Kernel/FastPath/Prepared/CompiledLayout/UniformArena and batch recording/copy/
submission contracts remain byte-identical. Checked byte offsets and native
narrowing, fresh groups/retained chunks, empty-launch admission/counter
precedence, folded dispatches, lazy preparation and post-submit callback order
remain as inspected. Current accessor/grid/result adapters and materially
affected unchanged PassParameter/PassParameters, Buffer, definition/cache,
mesh/BVH/physics/XPBD and retained hair/armor/character-creator ports retain
their existing relationships. No new source/API consumer was introduced by
the three-path S5 delta.

Existing tests, fixtures, helper bodies, assertions, workflow and hosted
evidence steps are unchanged from the preceding checkpoint. The owning guide
continues to distinguish historical observations from current acceptance and
retains browser, hardware, performance and rendering validation limits.

## Required unresolved dependencies and limits

#780 still owns required checked count/shape/byte producers, source/device
cache families, KernelCache documentation and GpuMesh caller integration.
Retained #774 hair/body/armor/M06 families remain required. Known bare-array
KernelBatch::dispatch consumers still conflict with the WorkgroupGrid
signature, including hair guide/grow bodies and armor/character-creator ports.
Those are unresolved coordinated dependencies, not accepted deferrals or a
reason to restore primitive forwarding APIs. No competing types or independent
claimed-function repairs were introduced by this reviewer.

Only source reads/searches and fingerprint inspection were used. No local
checks, builds, tests, scans, generators, runtime execution, source edits,
new tests/fixtures/cases/helpers/assertions, push or hosted CI occurred. Only
this ignored report was written. No external review was posted.

After the owner dependencies are integrated, repeat all thirteen full audit
lenses and independent Code Review, Rule Review and Comment Review at the
same final pushed revision with required hosted checks. This local checkpoint
review cannot satisfy that final-revision requirement.
