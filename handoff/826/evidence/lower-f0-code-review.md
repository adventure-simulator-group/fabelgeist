# Lower successor Code Review

Frozen HEAD `f0a98e58d54b673e017cf5ae633e9b6c46b5b609`, tree
`8620087c910f213069bf8b257b2d0af6d2e5ad78`.
Parent `d4491e9de215ba1b607fd2989cbfd5e3cbbdd21d`; pinned main
`990430b77a9f63548ffcb642074185f61ddb3322`.

All12 changed files read in full; 61 source identities all match frozen working
bytes. Source-map SHA-256
`492e18146431a725d7ec231017328f4eb93f01b3ed0e13c6d121825a30655d66`.

No new actionable owned receiver finding identified. LB8 is addressed in source.
This is a receiving checkpoint only: required peer/caller/lock integration
remains incomplete, and there is no compile, lower-family, all13, PR or final
clearance.

## Range proofs

Logical 3, physical 4: MappingExtent ceil(3/4)*4=4<=4; read target (native
MAP_READ or aligned staging copy) maps0..4; mapped_range uses same retained
extent; host Readback<u8> copies3, ignores fourth byte, releases after view
drops.

Logical 4, physical 6: MappingExtent4<=6; original MAP_READ request/view0..4,
not full6. SDK rejects6-byte map but accepts4. Prior LB8 is addressed by
retained extent.

Logical 0, physical 0 or nonzero: Logical<=allocation admission then
MappingExtent::for_prefix returns None; host Readback admission still rejects
ZST and reserves0 non-ZST; copy_from empty returns empty without native
slice/map/copy.

Logical oversized or alignment overflow, physical insufficient: Logical>physical
fails before allocation. ceil alignment checked_mul overflow becomes
MappingExtentOverflow. Rounded prefix>physical becomes ExtentExceedsAllocation
before host/GPU work or native slicing.

Logical partial T or host capacity, physical sufficient for rounded range:
Readback validates whole nonzero T, checked host narrowing and actual
try_reserve_exact before any native map/view; source truncation before unsafe
copy, AnyBitPattern+Vec<T> alignment and set_len-after-copy preserved.

## Revalidated contracts

All12 changed files were read in full, including unchanged portions. d449
remaining native/staging/metal/error ports were freshly source re-read or
explicitly exact-hash revalidated, without importing a previous pass as
acceptance.

Buffer allocation observes immediate Validation in definition; queue write
observes actual paired-queue sink and keeps bytes borrowed only through write.
Context exact-device/queue preconditions now documented on
new/write/write_at/from_upload/from_parts. Public native/size authority
weaknesses are explicit preconditions, no false runtime provenance proof.

Native aligned staging copies MappingExtent bytes admitted against physical
descriptor, not merely logical bytes; no returned padding. Actual native
COPY_SRC/usage/same-device/bounds/alignment failures still reach context.submit
finish/submission scopes. No premature zero skip added to general copy API.

Mapping request owns one checked extent used for both map_async and
get_mapped_range. Native callback may be synchronous; it only sends and request
ownership is installed before scope outcome inspection. Captured map error drops
request after map scope is popped, so cleanup scope cannot mask original.

Fallback Drop locally opens native-only cleanup Validation scope, including
documented borrowed-context exception; unmap resets API map state and Idle
NotMapped is discarded as secondary. Guard pops immediately on Drop with no
poll/wait/await. Successful release disarms before observed unmap, and mapped
view is dropped first. Browser cancellation drops receiver+request, unmaps and
leaves callback send harmless.

Native request complete retains Wait/Poll actual causes; no Pending cancellation
point exists inside blocking native poll/thread yield. Browser awaits callback
and has real cancellation point; no ErrorScopeGuard retained across it.
Configured device-handled panic before request creation or external alias
interference are not promised recoverable errors.

Host Readback accepts BufferByteLength directly and retains typed
host/address/allocation/map/poll/channel/native causes through source chains;
original cases/oracles remain after only six constructor operand adaptations. No
new test executable code or assertion was introduced.

WorkCompletionError retains real marker wgpu Error, PollError and oneshot
Canceled via source(). Native submitted_work_done observes empty marker under
existing policy then waits exact SubmissionIndex with timeoutNone; no completion
wait added to submit itself. Observed marker Err returns before poll; poll Err
before Ok. Borrowed/native policy still makes no synchronous marker validation
verdict.

Browser completion registers SDK queue callback and awaits receiver without a
poll timer. Pinned WebGPU backend2721-2731 invokes callback on promise success
OR rejection, logs rejection and exposes no cause. Current docs honestly
constrain Ok to callback delivery. Dropping wait only drops oneshot; no mapping
resource is held to clean.

Context creation retains adapter/device/surface native sources; Canvas JsValue
remains public payload because it is not std Error. Pinned wasm-bindgen0.2.126
non-atomic Send/Sync and existing fragile non-atomic wgpu policy remain; no
atomics support claim. Initializer delta removes only old target-format comment.

Staging session identity, checked index, checked byte multiplication,
zero/ZST/short prefix and typed allocation contracts unchanged and freshly read;
GenerateError::Readback/Staging/Gpu sources remain. Facade Result alias
preserves same public carrier and build_in/nearest propagate submit before read.
Metal source owner unchanged and peer APIs remain required.

## Required dependencies

D780-canonical — REQUIRED unresolved root integration. facade.rs imports
QueryCount/PointSelection/MeshVertexCount; metal imports
BufferElementCount/Stride and MetalTextureSize; staging record/read new
signatures. Exact peer42d/2bb7/90b/6f signatures were read and match receiver
uses. Root still has old canonical files, so frozen receiving root cannot
compile alone. This is a source proof of open integration, not acceptance or
suppression of mismatch.

D780-staging-callers — REQUIRED unresolved producer migration. part.rs
finish/PartSlots, body.rs finish, bracer.rs read_async, breastplate/mod.rs
read_async. Old callers pass tickets by value, assume Copy/get and infallible
status/prefix; now need borrowed Clone tickets, explicit counts and ?. All
actual bodies inspected for these relationships. No shims or synthesized missing
bytes.

D780-metal-and-application — REQUIRED unresolved whole caller/error/cache
family. metal/bake.rs textures/bake/scratch/dispatch and existing metal/tests;
creator armor_gpu/metal_gpu cached String. Root receiver changes to anyhow,
MetalTextureSize, count/stride/InvocationCount need agreed owner migration
through bake/material/Engraving/application. Cached errors must remain
structured; ignored bake submit is still old source and cannot be declared
repaired here.

D780-manifest-lock — REQUIRED unresolved lock integration. armor Cargo.toml vs
Cargo.lock armor dependency entry4844-4862. Root added anyhow and
fabelgeist-mesh; frozen lock armor entry still omits both. Must reconcile in
combined integration before locked hosted checks; no local Cargo execution run.

D774-M06/F19 — REQUIRED connected remaining family. retained fitting/cache error
and extraction output owner obligations. Human authorization resolved. Exact
lower receipt does not clear fitting status/frame/cache source erasures,
extraction completeness or all owner producer families.

D780-completion-callers — REQUIRED explicitly assigned producer/publication
migration. shell.rs step_interleaved:294, project_host_contacts:369; garment-fit
lib.rs step_and_wait:602. All3 distinct complete control flows inspected.
Current old calls discard new WorkCompletionError and continue Ok/loop
publication. #780 explicitly owns these complete functions; migrate propagation
and dependent state at owner before integration. This is known pending caller
work, not a new completion receiver implementation defect.

## Existing fixture identity

{"constructor_operands_adapted": 6, "context_inline_tests_exact": true,
"metal_fixture_exact": true,
"same_old_fixture_body_after_operand_adaptation_and_whitespace": true}.

The six host Readback constructor operands now wrap the exact same literals in
BufferByteLength. After that operand adaptation and whitespace normalization,
every old case/value/oracle/assertion is identical. Fixtures were
source-inspected only, never executed.

## Limits

- No new actionable owned receiver finding identified within stated coverage.
  This is not complete lower family, compile, root all13, wholePR or final
  clearance.

- Root canonical/caller/manifest-lock integration remains REQUIRED; old
  unintegrated APIs do not compile alone, and this report does not accept them
  as complete.

- No source edits, project
  execution/check/build/test/scanner/generator/runtime/push/CI. Existing fixture
  constructor operands only; no new tests/cases/helpers/assertions/doctests.

- Completion browser hides SDK promise rejection cause; native observes
  Validation only under blocking policy. Other native classes/device loss/global
  health follow configured SDK handling.

- Exact-creating-device/paired queue and exclusive external mapping
  preconditions remain explicit; no arbitrary-alias guarantee or fake public SDK
  map-state query.

- Final integrated source requires full13 waves, all three independent reviews
  and required hosted checks on exact same final pushed revision.

