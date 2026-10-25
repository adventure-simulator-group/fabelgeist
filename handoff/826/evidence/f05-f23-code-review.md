# Independent F05/F23 Code Review

No additional actionable owned correctness defect was found at this checkpoint.
F05's retained byte metadata and F23's provided-value packing protocol are
source-consistent with the pinned base. The earlier c798 packing-specific
approval was incorrect: expected-shape gating narrowed accepted inputs and
changed bytes, and direct nonstruct uniforms were not named destinations in
the base. Those earlier conclusions are withdrawn and were not used as evidence
for this review. Required owner integration and hosted validation remain open;
this is not whole-PR or final clearance.

## Exact source and independently checked fingerprints

- Reviewed HEAD: `3ff23fc9a47fe32fe514747a17585606ea001475`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Git reported a clean working tree. All 73 changed files matched committed
  HEAD bytes. The c798-to-HEAD repair delta contains ten paths.
- SHA-256 of the full binary main-to-HEAD diff:
  `81b85a4585979457fac3948d757f834e92774ac6465082e63420cf20a3319ef5`.
- SHA-256 of the independently computed changed-file path-to-SHA-256 map,
  encoded as sorted compact JSON:
  `ca71e2b5edaf192d2fa47f5dfc50ad93afa981a98500546b4a6b3898c774cbb8`.
- Every current `f05_f23_post_repair.files` fingerprint matched. The older
  `f05_f23_implementation.files` receipt is stale for bind_group/uniforms.rs
  and texture-views.md; it was not treated as the current snapshot.

Read the actual base inline static packer in compute/pass/mod.rs and the base
cached packer in kernel/fast.rs, then compared the current full producer,
retained metadata, allocation and both packer families. Read the incoming F05/
F23 evidence and reopened all-thirteen pre-repair ledger after source
inspection. The wider array/nested/member-prefix/direct-zero behaviors were
examined as existing source contracts; no new tests or cases were authored or
executed. Other full-scope and affected-graph contracts were independently
revalidated against current identity and actual producer/recipient ports.

## F05 byte producer and retained-state verdict

`gpu/shader.rs:10-14,38-45` now stores member offset as BufferByteOffset and
member size/group extent as BufferByteLength. These are the existing shared
owners; zero remains representable. The Naga producer in
reflection/uniforms.rs:25-43 widens/admit its actual native u32 offsets, type
sizes and struct span once, then retains the canonical leaves.

Static allocation and packing consume those leaves directly. FastPath retains
the group's byte length and member records rather than extracting a primitive
and re-admitting an equivalent byte field. Its scratch-capacity comparison is
between byte lengths. Extraction occurs at actual host-slice addresses, native
buffer/minimum-binding descriptors and native alignment/stride encoding ports.
No second byte type or parallel authority was introduced.

Host-address narrowing uses usize::try_from. `UniformHostAddress` retains the
actual TryFromIntError, and ComputePassError::source at error.rs:95-101 returns
that concrete cause. The original conversion is not stringified or replaced by
a fabricated error. BufferByteLength supports the required From<usize> and
From<u64> conversions; Default remains the canonical zero used by reflection.
No stale raw-field consumer or removed UniformValueShape reference remained in
the searched GPU/compute source.

## F23 baseline behavior comparison

### Reflection and accepted metadata

Base reflection retained each immediate member of a top-level uniform struct
by name, offset and native type size, without requiring a particular declared
shader kind. The current reflection producer restores that exact inventory.
Array, nested-struct, integer-vector, rectangular/wide and other validated
member kinds are not filtered out merely because their declared type differs
from the supplied PassParameter encoding.

For a direct nonstruct block, base reflection retained its native extent and
an empty member dictionary. Current reflection/uniforms.rs:36-43 does the same.
Static packing therefore leaves it zero filled; the fast writer loops over no
members and likewise leaves zero bytes. A supplied parameter keyed by the
global's name is ignored, rather than becoming a new destination or causing a
fast missing-parameter error. UniformValueShape, its admission helper and
UnsupportedUniformType were removed instead of preserved as compatibility
metadata. Multiple used uniform blocks in a group remain an explicit existing
unsupported-interface preflight, separate from uniform member type handling.

### Static provided-variant bytes and zero behavior

Base compute/pass/mod.rs selected the supplied PassParameter variant and
checked the whole block's bounds. It did not impose the member's declared
type or its individual span as a write barrier. Current
bind_group/uniforms.rs:26-52 again chooses the supplied variant, and the old
guard policies are preserved:

- Number narrows to native f32 bytes; Unsigned preserves the native word,
  including the unchanged From<i32> adapter's bits.
- Vec2/Vec3/Vec4 write their supplied prefix if the complete supplied vector
  fits in the block. They may write across the reflected member's span, just
  as in the base. The existing F23 vec4-member/Vec2-value counterexample now
  takes the Vec2 writer, restoring its first eight bytes and remaining zeros.
- Mat2 uses the existing 16-byte span/eight-byte column stride. Mat4 and
  Transform use the existing 64-byte span/16-byte stride and column-major
  traversal. Whole-block-not-fit outer guards still skip these writes.
- Missing parameters and unsupported provided variants are skipped after zero
  initialization. No supplied-value/declared-type mismatch gate remains.

The same protocol applies to immediate nested and array members: their native
offset/span facts are retained, and the supplied value selects the existing
byte writer. There is no new recursive schema packing or rejection path.

### Mat3 overlap and newly typed failure

Base Mat3 encoding uses the reflected member size as its outer span and
`member.size / 3` as its column stride. Each component is written in column
order then row order. Current write_matrix at uniforms.rs:82-119 computes the
same address `start + column * stride + row * sizeof(f32)` and uses the same
order. Successful overlapping in-bounds component windows therefore retain
their original final bytes; the refactor does not impose a member-span barrier
or normalize the historical stride.

The outer span-not-fit branch still skips the matrix. If an inner component
window would exceed the entire block despite that outer check, the base slice
operation could panic. Current checked component addressing returns
UniformPackingOutOfBounds or UniformPackingOverflow instead. A local data
buffer can contain earlier writes when the error is detected, but pack_uniforms
returns Err and uniform_buffer::create propagates it before native allocation
or upload; there is no successful partial packet or externally committed
partial state. Checked host-address failures retain their concrete source.

### Cached provided-variant protocol

Compared current fast/uniforms.rs directly with the old inline function.
Missing parameters still fail, now through the canonical typed classification.
Number/Unsigned/Vec2/Vec3/Vec4/Mat4 still select their own byte encoding and
whole-scratch-block fit guards, irrespective of declared member kind. Mat2,
Mat3, Transform and other unsupported supplied variants still fail on this
narrower cached path. The removed shape check no longer rejects previously
supported shorter or differently shaped supplied values before those writers.
Offsets are narrowed only at the actual host-address boundary. Actual admitted
fast members belong to the bounded reflected block; no arbitrary retained
offset is admitted through a mutable pipeline reflection API.

## Remaining full-diff and affected-graph receipts

The other 63 scoped paths are byte-identical to the preceding snapshot; their
contracts were revalidated, not inferred from the invalidated shape verdict.
Current texture constructors/selectors/removal and generic/face/array/direct
consumer graph still preserve cached Arc identity, missing-state precedence,
alternate allocation/device admission, storage sRGB normalization, selected
view effective metadata and actual original native causes/requests.

Immutable shader/module/device and pipeline interface/cache authority, shared
compute-entry resource/sampling union, used Immediate rejection, sparse group
slots, comparison/multisample/storage roles, typed unrepresented resource
payloads and concrete source chains remain unchanged. The pipeline-admitted
shared assembler still reaches the corrected static allocation/packing graph.
The fixed converter layout, transfer shader, device checks, absence ordering
and native cause scopes remain intact. Native provider/borrowed/browser limits
remain documented.

Kernel/FastPath/CompiledLayout/Prepared/UniformArena and batch dispatch/copy/
submit retain checked byte slots, fresh groups, retained chunks, empty-first
admission/counters, folded dispatches, lazy preparation and callback ordering.
The F05 fast uniform-size change preserves the same native limits and stride.
Affected unchanged parameter/buffer, compute, definition/cache, mesh/BVH/
physics/XPBD and retained hair/armor/body-device ports preserve their source
relationships, pending the explicit owner work below.

Existing tests, fixture and oracle populations are unchanged, including the
existing helper API adaptations. No new assertion, helper, case, fixture or
doctest was introduced. The guide now states the actual provided-variant and
direct-zero protocol, including the intentionally permissive whole-block byte
bounds and typed former-panic behavior; the previous shape-preservation claims
are no longer presented as current behavior.

## Required dependencies and limits

#780's cache/source-producer implementation is in progress and must be consumed
without competing KernelCache/Kernel edits. Retained #774/F19 source families
and known bare-array-to-WorkgroupGrid receiving failures remain required.
They prevent connected source completion and are not accepted deferrals. No
source-complete or whole-PR pass is implied by this owned-source review.

Only source reads/searches, Git identity and fingerprints were used. No local
checks, builds, tests, scanners, generators, runtime, source edits, push or CI
occurred. Only this ignored report was written. All thirteen complete lenses,
three independent reviews and required hosted checks must agree on the same
final pushed revision after required integrations; those gates remain future
work.
