# Independent reflection-checkpoint Code Review

No new actionable correctness defect was found in the owned implementation at
this checkpoint. The reflection changes address the supported host shapes and
native resource roles consistently. Required #780 and retained #774 families
remain incomplete, including known consumer signature failures. This is not
final clearance or a source-complete verdict.

## Exact revision and independent fingerprints

- Reviewed HEAD: `2eb1129b0a8d09289aab9e7cc83bc3499a8a3d9e`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Git reported a clean working tree; the main-to-HEAD diff has 73 changed files.
- SHA-256 of `git diff --binary BASE HEAD`:
  `954f0bb4f160a4a4e11fd5c17fb831c0bc8296af4f4eec4d048d99b84dfcc8d2`.
- SHA-256 of the independently read changed-file path-to-SHA-256 map, encoded
  as sorted compact JSON:
  `2ba2877bdd20f5313c74cc8510664816b3361b0a4793329e2036bd2de682fcc0`.
- Every fingerprint in both ledger entries `reflection_repair_checkpoint` and
  `reflection_wave` matched independently, and both name this exact revision.

Read the complete successor delta and current reflection/type/packer source,
and reconciled it with the whole main diff and affected graph inspected in
this ongoing independent review. Unchanged evidence was revalidated against
current fingerprints and producer/recipient relationships, rather than taking
an earlier verdict as current clearance. The ledger records complete discovery
at the preceding checkpoint before the S1-S4 repairs. Read the new contracts
after inspecting source. This reviewer made no implementation changes.

## Reflection shape inventory and correctness receipts

### Actual resource usage and shared entry-point layout

`compute/pipeline/reflection.rs:20-64` parses and validates the same source
that shader admission uses. It includes a bound global only when at least one
compute entry point uses it. The retained layout intentionally covers the
union across all cached compute entry points, including resources used only by
a later entry point. Unused globals require no caller binding, including
unused unsupported payloads. The first compute entry remains the default;
alternate entry-point cache identity and immutable interface remain intact.

Checked the pinned SDK sources rather than assuming the analysis API:

- Naga 29.0.4 `valid/analyzer.rs:248-275` documents `sampling_set` as exhaustive
  for an entry point, including sampling through callees, and global usage as
  indexed by GlobalVariable handle.
- Its `FunctionInfo` index implementation at `326-331` and ModuleInfo's
  `get_entry_point` at `1298-1300` support the actual API uses here.
- Naga 29.0.4 `ir/mod.rs:812-840` defines Copy ImageClass and the matched
  Sampled, Depth, External and Storage variants.
- wgpu-types 29.0.4 `texture.rs:376-437` defines the used storage access roles,
  including ReadWrite and Atomic. Provider features still govern acceptance.

The float-filtering requirement is the union of image sampling pairs across
compute entry points. It is not inferred merely from image presence. A
load-only float image therefore receives Float { filterable: false }, while an
image sampled by any cached compute entry receives the filtering requirement.
Existing native feature/limit validation remains authoritative.

### Images, samplers and unsupported resources

Read the complete `reflection/images.rs` and parent admission match. Sampled
float/uint/sint and depth image roles retain their scalar class. Multisampling
and arrayed D2/Cube geometry are preserved. Storage formats use the existing
exhaustive Naga-to-wgpu mapping in texture/format.rs:158-201; no default format
or string classification is introduced. Access maps separately to atomic,
load-plus-store, store-only or read-only. Comparison samplers retain Comparison
layout entries; ordinary sampler entries retain their existing Filtering role.
The named SamplerBinding producer/recipient representation is consistent.

Used binding arrays, external textures, unsupported handle payloads and
unsupported bound address spaces produce typed UnsupportedBinding admission
errors with group/binding/kind. Storage buffer content arrays remain ordinary
Buffer bindings, distinct from arrays of resources. Unsupported arrays are not
silently flattened into one resource. Sparse layout slots and indexed group
binding remain unchanged, with device group-limit admission before allocation.

### Direct/flat uniform shapes and both packers

`reflection/uniforms.rs:9-60` admits one uniform binding per group. A flat
struct retains member names, Naga offsets and the complete declared block span.
A supported direct uniform retains the variable name and offset zero. A second
block in that group fails as MultipleUniformBindings rather than overwriting
the first block's metadata. Different groups still retain independent blocks.

`uniform_shape:63-109` accounts for every branch: 32-bit float scalars,
32-bit integer words, f32 vectors of length 2/3/4 and square f32 matrices of
size 2/3/4 are admitted. Nested/array values, unsupported scalar widths/kinds,
non-float vectors and rectangular/non-f32 matrices fail with a typed
UnsupportedUniformType identifying the group, binding and name. Validated
native types therefore reach packing with an actual supported host role.

`shader.rs:75-94` maps these shapes to the existing PassParameter payloads.
The unchanged `parameters/parameter.rs` From<i32> conversion stores the input
as Unsigned(value as u32). Admitting Sint32 and Uint32 as IntegerWord therefore
preserves their existing four-byte native bit representation, including
negative signed inputs. It does not reinterpret signed bits as a floating
number. Mat4 also admits Transform, as the static packer already did.

Static packing gates each supplied value by its reflected shape before
writing. It preserves the declared policy that missing or mismatched values
leave zero bytes. A wider supplied vector/matrix cannot overwrite neighboring
members. Existing scalar/vector/matrix/Transform byte order and Mat3 padded
column stride remain. The uniform extent is checked against allocation and
binding limits before packing, and unmapped allocation precedes upload.

The cached packer checks missing values and shape compatibility before writing
and returns the canonical ComputePassError categories. Its narrower matrix
support remains intentional and documented: Mat2/Mat3/Transform are rejected
by that path instead of silently leaving bytes or fabricating a conversion.
The bounded scratch buffer, fresh per-dispatch bind groups, slot allocation,
dynamic offset and native error propagation remain intact. The error-family
change also distinguishes missing buffer parameters from wrong buffer kinds.

The fieldless UniformArenaError derive retains all four Display messages and
the prior no-source Error behavior. The compute crate already depends on
thiserror; no dependency or trait-bound requirement was inferred from an
unavailable build result.

## Whole diff and materially affected unchanged graph

Revalidated the three whole-texture selectors/constructors, generic/face/array
admissions, controlled cache/allocation removal and direct copy/read/write/
clear/render recipients. Cache-first missing behavior, detached-cache sharing,
alternate native admission, device checks, storage sRGB normalization and
original native cause/request retention remain as documented. Selected view
format/dimension/device stays distinct from native allocation format.

The sRGB converter retains the fixed unfilterable float input layout, its
creating device, source/destination cache-absence order and explicit native
construction/record/finish/submit scopes. Its shader algorithm and workgroup
coverage remain unchanged. Source/device/module coherence, typed Naga/native
errors and immutable pipeline reflection/cache relationships persist through
the reflection refactor. Cache rejection and retry retain the actual native
source; borrowed/browser contexts retain their documented synchronous limits.

Read current Kernel/FastPath/CompiledLayout/UniformArena/Prepared and batch
dispatch/copy/submit relationships. Empty batch launches still precede
admission and dispatch-count increments; standalone empty launches still admit
parameters. Checked byte offsets and native narrowing, folded launch behavior,
lazy preparation, callback order and transparent count serialization remain.

Reviewed changed compute/accessor/grid/result adapters and affected unchanged
PassParameters/PassParameter, Buffer allocation/upload/binding, definition/cache
ports, mesh/BVH/physics/XPBD recipients and retained hair/armor/character-creator
call sites. All repository UniformMember construction sites are the admitted
reflection producer; no authored UniformMember initializer was left missing
the new shape field. All shape and sampler consumers were checked for the new
record/API relationships. Existing generic caller `?` conversions retain typed
causes instead of formatting replacement errors.

Existing texture tests, fixture values and sort's typed-zero adaptation retain
their cases and oracle expressions. No new tests, fixtures, cases, helpers or
assertions were introduced by this reviewer. The workflow, owning guide and
context comments preserve hosted-final-revision acceptance and documented
hardware/browser/performance/rendering limits. S4's comments state native
synchronization and fresh-group policy without claiming an unproved barrier
mechanism.

## Required unresolved dependencies and limits

The known consumer compilation issue remains: owner-held hair, armor and
character-creator bodies still pass bare arrays to KernelBatch::dispatch,
whose signature requires WorkgroupGrid. Representative locations are hair
guides.rs:223-241/292-303, grow.rs:577-597, armor anatomy.rs:164,
breastplate/kernels.rs:160, bracer.rs:237-241/261, metal/bake.rs:152-159 and
character-creator device_frames.rs:219/225. Rust does not implicitly convert
those arguments. These remain coordinated owner dependencies, not accepted
deferrals or a reason to restore primitive overloads.

#780's count/shape/byte producers, source/device cache families, KernelCache
and GpuMesh caller remain required. #774's retained hair/armor/body/engraving
families remain required. Their source and contracts must be integrated before
source completion or publication; this review did not duplicate their types
or independently repair claimed functions.

Only source reads, source searches and fingerprint inspection were used. No
local checks, builds, tests, scanners, generators, runtime execution, push,
hosted CI or external posting occurred. Only this ignored report was written.
The complete thirteen-lens wave and all three independent reviews must be
repeated at the final pushed revision together with required hosted checks.
This checkpoint review cannot satisfy that final-revision requirement.
