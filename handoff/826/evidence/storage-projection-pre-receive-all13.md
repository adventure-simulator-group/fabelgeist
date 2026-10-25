# Storage/projection receiving inventory

Root `954c388388adb45e19d917d61b7e24e7746616f0`; incoming `e2162708fcf3885193db434300ed6c2999ab7607`; base `990430b77a9f63548ffcb642074185f61ddb3322`.

Bounded production-owner discovery is complete. Repair and validation have not
started in this independent review. Required Min/Max authority, whole extraction,
fixture provenance and global integration acceptance remain incomplete. This is
not source compilation, whole-PR or final clearance.

## Actionable connected findings

### SP-CUBE-NATIVE (P1)

Initialized storage-capable cube, matching native context, shader_src containing
invalid WGSL or a shader returning the wrong store value type. view_2d_array
succeeds; raw create_shader_module reports an immediate native validation error
before context.submit has installed any scope. SDK default handling panics, or a
configured handler receives the error. Result<()> does not retain the actual
creation cause. Native layout/bind/pipeline failures have the same timing hole.

Required repair: Keep existing view-first admission; resolve effective storage
spelling next, before native construction. Extract the generated cube
responsibility into a focused module, retaining existing whole public method
ownership. Observe the complete native create+record closure with
context.observe_validation, always drain before checking the outcome, map actual
native cause at the established ComputePassError/native boundary;
finalize/submit through existing context.submit afterwards. No new blocking
wait. If any fallible application work remains inside the observed body,
preserve its earlier primary error and a secondary observation failure using the
existing resolver rather than early-returning or masking.

### SP-REDUCE-EMPTY (P2)

New ReductionPlan admits zero complete Buffer records and Reduce::execute
returns that logical input unchanged. Buffer::read returns an empty Vec for
logical zero; Root954 scalar number consumers index data[0] and panic. Canonical
e216 versions use first().ok_or(EmptyReductionResult) in all three scalar
branches.

Required repair: Obtain and consume the agreed complete canonical min/max
consumer functions/files with Reduce/error; do not leave the old consumers or
invent another empty-result owner.

## Thirteen lenses

### existing-type-owners

Reuse PixelFormat native storage spelling, ResourceBaseType classification,
TextureFormat/effective selected view, CoordinateShape,
BufferElementStride/Count, WorkgroupShape/Grid and existing native error owners.
No duplicate storage table, flat count, coordinate strings or native cause
wrapper.

Canonical owners source traced. Root builder already consumes the agreed typed
base_type and fallible output binding, pending actual owner receipt. Existing
ResourceBaseType::element_size is not a native array stride and new programs
correctly stop using it for dispatch. Native Cube cause gap is actionable.

Acceptance: Actual selected storage format matches native declaration/layout.
All five ResourcePipeline controlling resources consume reflected stride/shape.
No fallback rgba32float, no leaked base_type_str, no substitute types.

### magic-numbers

Every numeric literal/field is assigned to native WGSL text/SDK descriptors,
texture pixel metadata, byte/layout owner, dispatch geometry or exact fixture
arithmetic; native representations stay at those adapters.

Generated binding ordinals, lane selectors, workgroup sizes, cube face formulas,
zero padding, integer/floating source arithmetic are authored WGSL/SDK
protocols. Native stride/count bounds are typed. AdvectSignature.dimension is
unused public descriptive metadata; it does not feed dispatch or authorize a
shape. Stencil dimension flags remain parser-produced native shader-template
selection metadata; this receiving inventory does not declare a new broader
Stencil-policy migration complete.

Acceptance: No change in uniforms/time/face/order/shader literals. No raw byte
stride/count fallback in migrated dispatch. Actual SDK counts converted once at
native source; valid zero not narrowed away.

### magic-strings

Classify language spellings at parser boundaries, carry canonical values in
state/recipes; leave authored WGSL, identifiers and diagnostic prose open at
their real text boundaries.

Coordinate strings become CoordinateShape at parse. ResourceBaseType::Custom
remains open for general buffer structs/aliases; TextureValue rejects
unsupported texture shapes explicitly. Source strings/regex/template tokens and
native labels are actual code/SDK boundaries, not mutable presentation-based
error branching. Cube unsupported storage string fallback removed by canonical
owner.

Acceptance: No to_string/reparse classification of canonical base type. No error
branching on Display. No invented validation of open WGSL names, Custom buffer
types or source text.

### producer-propagation

The admitted source/device/recipe, native shape/stride and address requirement
must reach every retained program and dispatch consumer intact.

Original manifest incomplete; supplemental and nine-file package now provide
direct owners. Required Min/Max canonical empty consumers are outside those
manifests. Extraction is explicitly held whole, including lib exports; no
individual extraction/wgsl receipt. Global producer/caller graph remains
incomplete.

Acceptance: All direct imports/module declarations resolve from the receiving
graph. New API return types propagated through actual caller bodies. No
temporary raw adapters/aliases or missing owner definitions.

### redundant-authority

Retained source, descriptors, secondary bindings, native layout and launch must
have one coordinated authority; caches may retain immutable derived state only
with complete identity.

No independent mutable layout setter or raw size tuple remains in new
ResourcePipeline. Sorted secondary recipe identity makes HashMap iteration order
irrelevant. Source cache includes source+actual device+validation policy and
only publishes successful builds. e216 pipeline reference cannot overwrite Root
transient error/removal semantics. Caller resource identity is separately
admitted by binding/device owners.

Acceptance: Source edits after a cache hit produce a new key. Native transient
progress stays uncached and removes stale unchecked entry. Shared layout/shape
always belong to selected pipeline; no sizeof approximation.

### type-ownership

Keep canonical shared GPU leaves in GPU; generated
ResourcePipeline/SecondaryResources in compute; PixelFormat in color; no
dependency inversion or competing owners.

Owner directions source verified. Stride adoption explicitly confirmed by
coordinator. Root native Buffer/Context/pipeline/transfer and whole
TextureBinaryOp builder remain protected. Cube two whole functions are now Root-
owned; focused extraction should own that responsibility without duplicating
source/format/error authority.

Acceptance: Each semantic owner defined once. GPU lower crate does not depend on
compute algorithms. Mixed-file composition preserves unrelated Root source byte
identities.

### type-granularity

Distinguish texture coordinate/index/population projection, native record
stride/count, resource kind and selected format; do not impose flat-volume
limits on independent texture coordinates.

Coordinates need no flat cap; LinearIndex admits largest population-1;
LinearPopulation admits population itself. Native allocation feasibility and
source value shape are separate. No arbitrary positive logical extent policy is
added. ResourceFetch code+requirement pair prevents forgetting flat projection.
Open Custom buffers preserved. No extra role wrapper proposed.

Acceptance: Population 2^32 is representable for LinearIndex but rejected for
LinearPopulation. Zero textures/empty grids retain actual lower outcome. No
texture-volume cap for coordinate-only shader paths.

### admission-contracts

All public constructors/defaults/source parsing/format generation/native
creation must admit actual errors without bypass, including
missing/cached/device/effective-format states.

Canonical semantic admission traced, including direct public descriptors which
can still be rejected by actual native interface. Unsupported storage has no
fabricated native source; it owns actual format. SP-CUBE-NATIVE requires repair
before receipt acceptance. No stronger alias/device policy invented for public
native resources.

Acceptance: Default/missing Cube returns its current view/allocation error
before storage lookup. Wrong device precedes new native create; unsupported
spelling precedes native pipeline. Native failures retain actual causes;
browser/borrowed outcomes documented and unchanged.

### invariant-operations

Checked address/extent/stride arithmetic must preserve admitted values through
mutation, narrowing and cache publication, with valid zero and existing order.

u64 three-axis product checked; actual u32 narrowing owns TryFromIntError;
population-1 saturates only for valid zero. Row-major products bounded by
admitted population after shader coordinate guards. Count*stride u32 operands
fit u64; trailing bytes preserve complete-record floor. Reduce scratch same
device and exact logical extent; number projection needs SP-REDUCE-EMPTY. Native
binding logical/physical and per-axis limits remain authoritative, not replaced.

Acceptance: No truncating usize/u32 flat count in affected new owner methods.
Failures do not publish successful source cache entries. No scope left pending
on early fallible returns. No new zero-return shortcut that suppresses native
admission.

### aggregate-states

Program+layout+shape, source+recipe+device/policy, fetch source+address
requirement, coordinate name+shape and selected native view facts must stay
coherent.

All coordinated constructions read. ResourceDescriptor logical format is
retained for semantic resource identity; actual storage generation explicitly
selects linear counterpart and Root binding selectors verify the concrete
selected native format. Cube must use effective_format consistently for WGSL
spelling, native layout and selected array view. Public signature metadata is
parsed anew for builders; caller mutation of a returned signature does not
mutate a cached program.

Acceptance: No mismatch between actual storage spelling/layout/selected view.
Recipe/source changes cannot reuse stale layout. Complete candidates publish
only on successful native/host admission.

### enum-shapes

Closed semantic choices must carry the relevant data and retain independent
axes; do not turn every predicate/native flag into a redundant state machine.

Existing canonical enums own meaningful payloads. Optional index/size parameters
are genuinely independently optional. Numeric scalar/vector shape and
texture/buffer resource remain distinct. Stencil/Stream booleans in parsed,
immediately consumed native source signatures are not treated as unchecked
retained program admission; a broader signature-policy redesign is not silently
claimed complete here. Proposed Cube target enum may carry CubeFace and derive
native layer/group counts at its explicit GPU adapter; no new count alias
needed.

Acceptance: No target flag plus separately meaningful face field. Keep Face
exact layer vs AllFaces six-layer distinction. Preserve independent parameter-
presence/primitive sampling policies.

### derivability

Replace mechanical trait boilerplate only when an available derive preserves
exact representation, bounds, Display and cause semantics; retain semantic
conversions and source identity.

StorageFormatError is mechanically equivalent to thiserror::Error with error
attribute `texture format {format:?} has no WGSL storage format`, retaining
Copy/Clone/Debug, private format and no source. Neither pinned GPU Cargo
declares thiserror, while compute does. This is an optional
dependency/boilerplate simplification, not a nonexistent derive or a native-
cause bug; keep manual implementation unless coordinator deliberately adopts
that dependency. Coordinate/native classifiers and checked conversions are not
derivable. Other manual cause implementations preserve actual sources and exact
Display.

Acceptance: No added source cause for an Option-based unsupported-format
condition. Exact Display/source/traits preserved if a derive is selected. No
unchecked derive replacing admission or opening private invariants.

### behavior-preservation

Preserve successful shader arithmetic, resource/zero ordering, errors/caches,
custom buffer values and fixtures; distinguish intended unsupported-path repair
from unrelated numerical redesign.

Intended repairs: unsupported storage no longer silently names rgba32float; Vec3
texture values now use .xyz and padded vec4 construction; coordinate
requirements admitted without narrowing coordinate-only texture populations.
Preserve old successful custom buffers, native shader schedule, clamp/wrap
formulas, zero/trailing record semantics and F05/F23 provided-variant packing
policy. Cube scope remedy changes error retention only. Existing external
fixture modules are not imported by the production packages, but final exact
original fixture/API-adaptation provenance remains required, not accepted merely
from packaging claims.

Acceptance: All old successful formulas/order/WGSL and numerical POD/time bytes
identical except explicitly intended shape/source generation corrections. No
tests/cases/helpers/assertions added. Root lower source preservation hashes and
final source reviews/hosted gates on one final pushed SHA.

## Receiving boundary and unresolved dependencies

All original sixteen files, eight supplemental files and nine complete recipe
files were read, alongside exact source dependencies recorded in the JSON.
Original manifest omissions are explicit. Compose GPU module/stride exports and
compute lib selectively; reference pipeline/Buffer files have no overwrite
authority. Keep the complete extraction family held, including its lib exports.

D-MINMAX: required unresolved receiving authority. Peer whole-function/file
authority and exact receipt, then coherent receiving change.

D-EXTRACTION: required whole-family integration held by coordinator. Do not copy
extraction/wgsl or whole compute/lib exports; complete separate canonical family
inventory/ownership before receipt.

D-FIXTURE: required explicit original-main/#779 provenance remains unresolved.
Exact published base identity/adaptations account before final graph acceptance;
no authored tests.

D-GLOBAL: required broader producer/caller/manifest-lock integration and final
same-pushed-SHA gates pending. No source compilation, hosted CI or final
clearance inferred.

## Validation limits

All16 original files and all8 supplementary files freshly read completely,
including full Cube context although only two functions have authority.

All9 complete-recipe files freshly read, seven GPU reference owners and both
original references freshly read completely. buffer/mod context header/native
body source was read as context only; this report does not authorize wholesale
overwrite.

Selected immutable Root current lower owners/functions read freshly as recorded;
canonical counterparts plus full relevant diffs are separately labelled rather
than overstated as fresh Root reads.

Held extraction shader/table/advancing-front/full external mesh graph and
original published fixture provenance are not accepted here.

No project checks, scans, builds, tests, runtime, generators, source edits, new
tests/cases/helpers/assertions, push, CI or external posting.

No source/compile/whole PR/global13lens/final clearance.

The JSON is the exact source/hash/manifest account and contains every lens’s
contract, source evidence, disposition, repairs, dependencies and acceptance.

## Explicit connected closure amendment

The canonical BoundaryMode definition was read, but its source authority is a
required unresolved receiving dependency. Compose only the agreed boundary,
resource_pipeline and resource_recipe declarations/exports; never copy the
whole compute lib/prelude with unreceived extraction/sort/mesh exports.

SP-CUBE-NATIVE also covers the unchanged blit native creation phase and the
fast-copy encoder/record phase. All complete bodies were read. Blit creates
shader/sampler/layout/group/pipeline objects before submit; those immediate
errors have the same gap. Copy validation may remain deferred until finish;
keep context.submit responsible for that phase. Observe the complete native
create/record phase without changing copy selection, view/device admission,
face/format/layer, or native timing. The JSON records the exact distinction.
