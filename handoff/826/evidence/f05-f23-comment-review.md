# Provided-value uniform checkpoint Comment Review

Status: **no new owned comment mismatch; owner integration and final validation
remain incomplete**. This is not final clearance.

## Independently verified revision and source identity

- HEAD: `3ff23fc9a47fe32fe514747a17585606ea001475`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Commit tree: `46f792a260ecf1fda3c2ebb60774e0ee9bc501ea`.
- Working tree was clean on entry and remained at this revision throughout
  review. The ignored report is the sole authored artifact.
- Independently inventoried the ten changed paths since the prior local
  checkpoint `c798fa504db9bc6fd82695887e285751846dbdfc`, read the complete new
  guide and the current changed/moved comment/consumer graph, and compared the
  relevant original implementations directly from pinned main. The rest of
  the wider tracked graph retains identical source across that comparison.
- Applied the Comment Review skill and governing repository rules read during
  this continuous review sequence. No earlier clearance was inherited.

SHA-256 fingerprints of all ten changed paths at review time:

```text
d2cd499fad77e234828d620839107c91a349c88ae333e64094cc6ad924065d68  crates/fabelgeist-compute/src/kernel/fast.rs
0d9175592d18179ab5b8ae11985ce9ca7623d800be55ba9a1db8c08b3c65cd76  crates/fabelgeist-compute/src/kernel/fast/uniforms.rs
6531a6cac487b5a8649caeb8835197c208ba8b19ea8c13e152f686b0f1404a8f  crates/fabelgeist-gpu/src/data/gpu/bind_group/uniform_buffer.rs
2301030d44c5a2b26819c3612a19732a7bfc20e6837ab7ddc0a88f96ddee06d9  crates/fabelgeist-gpu/src/data/gpu/bind_group/uniforms.rs
5c4cf07dee49fd2f429712cd22b4d91b7c19b447efcd18b246fb1d64fc30279a  crates/fabelgeist-gpu/src/data/gpu/compute/pass/error.rs
707cf73485c1b7b28bd3e54cc5818fae51b67e4994ded26afe4f16f7ce5aaef4  crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/error.rs
d6d50cd02eba48905f36c9fa34512daa36e094446c2baa58f1163c57d5ae3c68  crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection.rs
7c79e1c26e7c308c8e8a8e041db1764eeb135a9b2ac7a034c72cc979543d536c  crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection/uniforms.rs
5600af987b9f0edf639896585aa820fc98d351daa64b81c9e84d8a51222a81b7  crates/fabelgeist-gpu/src/data/gpu/shader.rs
02b9c6fe5419780c4da5b6a10cfc7ace0c852866cc7c687336ee8a91a3190c25  crates/fabelgeist-gpu/texture-views.md
```

## Withdrawn shape-preservation premise

Earlier reports' expected-shape contract must not be used as evidence that the
refactor preserved the original packing behavior. The actual pinned-main
`ComputePass` packer and `kernel/fast.rs` packer select the supplied
`PassParameter` variant and check whole-block capacity; they have no expected
shader-shape gate. Pinned-main reflection records immediate struct members;
direct nonstruct blocks have a native extent but no named member dictionary.

I read those original paths directly in this review. The current removal of
`UniformValueShape`, its reflected field/gates, unsupported-declared-shape
admission and direct global-name parameter mapping is consistent with that
provided-value protocol. The current guide describes the revised contract;
the earlier shape-based preservation conclusions are superseded.

## Current packing comments and guide verified against code

The guide and `bind_group/uniforms.rs` now accurately describe initial zero
allocation followed by ordered writes selected by provided value variant.
There is no per-member zeroing. If an earlier wide supplied value overlaps a
later member, skipping that later member leaves those earlier bytes in place.
Neither the guide nor the new comments promise to restore those bytes to zero.

Array/nested struct members remain named immediate destinations. Their declared
shader shape does not select the writer or restrict its span: the supplied
variant and whole uniform block do. The current reflection's nonstruct branch
retains the native extent with an empty member list, so an extra host parameter
keyed by that global name cannot populate it. Static and fast paths consequently
write zero-initialized direct blocks, as documented.

Static scalar/vector/fixed-matrix encodings preserve the established component
order and outer block-window skip policy. Mat3 uses the reflected member span
and `span / 3` column stride rather than a newly imposed canonical stride. This
matches the original successful writes even for a supplied matrix value whose
destination declared a different type. Where that Mat3 rule admits the outer
window but a later component window would exceed the actual block, the current
implementation returns `UniformPackingOutOfBounds` instead of slicing beyond
the block. Checked arithmetic has its own typed overflow outcome. The guide
correctly calls out this bounded panic-to-error change rather than claiming
every formerly failing input behaves identically.

The cached packer retains its original provided-variant cases and narrower
matrix set. Missing or unsupported supplied values return canonical typed pass
errors; it has no new declared-shape gate. Whole-block bounds still control its
supported writers. Its comment refers to the supported supplied kinds and does
not promise broader matrix support.

Reflected offsets, member spans and group extents now use `BufferByteOffset`
and `BufferByteLength` through reflection/static/fast state. Native byte
addresses/component strides are explicitly identified at the host slice
boundary. `usize::try_from` failures preserve the concrete
`std::num::TryFromIntError` through `ComputePassError::UniformHostAddress`, and
`Error::source` returns it. The guide's concrete-conversion-cause statement is
therefore accurate. Native validation errors retain their separate SDK causes.

Signed/unsigned host input representation is unchanged: the existing `i32`
adapter stores `Unsigned(value as u32)` and both supported integer writers emit
that word's native encoding. Removal of the shape enum does not introduce a
new signed host variant or numeric reinterpretation.

## Whole guide and materially affected graph

Re-read the full current guide with the static assembler, allocator, fast
packer/state, reflection admission and typed errors. Its changed packing
paragraphs match the behavior above. No new owned wording remedy is requested.

Reconciled its unchanged claims with current source as well:

- Shared reflection still unions used resources and sampling sets across
  compute entry points. Unused globals impose no binding requirements; used
  immediate data is rejected before binding filtering/native reflected layouts.
  Comparison sampler, multisampling, dimension and storage access roles remain
  represented, with provider limits/features governing native acceptance.
- Multiple used uniform blocks in a group remain unsupported. Uniform array
  values are correctly distinguished from resource binding arrays/external
  texture payloads that the named interface cannot represent.
- Immutable source/module/device and reflected pipeline provenance, concrete
  available SDK error causes, complete observed native cache outcomes and
  device-handled versus observed validation remain accurately described.
- Selected texture-view format/dimension/device checks still concern the
  selected handle rather than allocation format. The three shape constructors,
  cached/absent ports, cache-first allocation-detachment behavior, alternate
  sRGB linearization, cube face/array geometry and source-less admission errors
  retain their documented relationships.
- The admitted-pipeline static assembler preserves sparse group indices and
  logical buffer prefixes. Static uniform limit checking still precedes host
  packing; unmapped native allocation and available native validation precede
  upload. The converter retains its device/absence/scope/fixed-workgroup policy.
- Per-batch uniform slots, fresh groups, empty launch admission/counter behavior,
  folded order and after-submission callbacks retain the reviewed contracts.

The changed/moved comment map includes shader/reflection/error owners,
texture/view/converter/context source, static and fast binding/packing and the
affected primitive/BVH/hair/mesh/physics graph. Those unchanged source blobs and
their existing fixture/example/workflow evidence were accounted for by exact
committed comparison, not a prior passing verdict. No original fixture input,
oracle, test case or compile-fail example was changed by this reviewer. The
guide retains its historical/software/hardware/browser/performance limits and
does not claim present hosted acceptance.

## Pending dependencies and validation limits

The accepted #780 owner repair to `Kernel` module/fast-field/cache wording and
its producer/cache source remains in progress. Their stale standalone-submit,
unconditional uniform-allocation and compile-once statements are not cleared
by this checkpoint. All retained #774 paths and F19 remain required integration
dependencies. This source review does not waive them or independently repair
claimed functions.

Only Git/source/document reads, the requested source fingerprints and this
ignored report were used. No source edit, local project check, build, test,
scanner, generator, runtime, new test/case/helper/assertion/doctest, push, CI
request or external posting occurred. No runtime or broad behavior-preservation
verdict follows from source review. Final independent reviews and required
hosted checks must still pass on the same final pushed SHA after all connected
families finish.
