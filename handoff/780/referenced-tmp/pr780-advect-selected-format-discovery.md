# Independent source investigation: Advect selected-format contract

The reported 2D sRGB defect is supported by actual producer, descriptor,
cache, binding and SDK source. The smallest canonical repair is Advect-local:
remove generation of IS_QUANTITY_SRGB and remove its manual transfer-function
branch and false textureLoad comment. Keep interpolating the textureLoad
results directly. No effective-format getter, alternate view, metadata rewrite,
parallel color implementation or cache mutation is needed for these whole
Texture2d/Texture3d inputs.

## Snapshot and limits

Root HEAD `b11d6697c5b2fc05ca59cf7497f3d6ab626580b5`; peer d215 HEAD
`8c600f727d1eecf21dd241f02739b825edef5000`, with actual current working bytes pinned below.
Read all of Root advect/mod.rs, ResourceDescriptor's declaration/binding/type
functions and ComputeSourceCache; read complete peer Texture2d/3d constructors
and views/selection and image-binding reflection/assembly. Consulted pinned
local wgpu-types/wgpu-hal/Naga 29.0.4 source. This is pre-repair discovery, not
clearance. No source edits, tests/cases/helpers/assertions, project checks or
external actions were made.

No Advect tests.rs or inline test module exists in either inspected checkout.
The parent's request assumed such a file and the parent acknowledged its
absence. No Advect fixture was invented or borrowed from unrelated image-copy
or tactical advection tests. Source inventory found only module/public exports
for this particular Advect API, no actual caller linking a nearby fixture.
Existing fixture provenance therefore supplies no runtime acceptance evidence
for this repair. Hosted checks and final independent review remain required.

## Confirmed color defect

Advect lines 62-73 derive one Boolean from quantity's nominal format and emit
IS_QUANTITY_SRGB. The sample_bilinear helper at lines 133-142 then manually
applies the sRGB-to-linear transfer curve to its already-loaded/interpolated
RGB. Lines 159 and 163 call the same helper for both velocity and quantity.

Actual SDK contract: wgpu-types 29.0.4 texture/format.rs:104-117 documents that
texture loading/storing performs its format conversion automatically;
UnormSrgb storage is encoded while shader values are linear, and Float performs
no conversion. Native Vulkan format mapping uses R8G8B8A8_SRGB/B8G8R8A8_SRGB,
not an application-defined format masquerading as sRGB.

Two concrete consequences follow without executing a new case. First, an sRGB
quantity texel is decoded by its sampled view and then decoded a second time by
the manual branch. Second, setting quantity to sRGB also applies that curve to
float velocity. A float velocity component 0.5 is changed to approximately
0.214; a signed component -0.5 enters the low branch and becomes about -0.0387.
This changes the displacement before sampling quantity, not only its color.
These are source-derived numerical consequences, not observed GPU results.

Minimal repair: remove the Rust format-derived Boolean and emitted constant,
remove the helper's manual conversion branch/incorrect comment, and return the
interpolated value. Preserve interpolation, boundary arithmetic, dimensions,
uniform/POD layout, workgroup policy, cache shape and 3D loads. The 3D entry
already uses direct textureLoad without this extra conversion, so this repair
changes only the erroneous 2D path's transfer handling.

## Producer -> descriptor -> cache -> selected binding -> store

1. Whole Texture2d/3d constructors allocate a linear underlying format with
   compatible view formats and create their cached view using the requested
   nominal format. For nominal sRGB they deliberately omit STORAGE_BINDING.
   Nominal linear storage textures keep a linear cached view. Constructors and
   defaults are the only whole-texture producers in the inspected source; view
   APIs create detached returned views rather than replacing these records.
   Cache-clearing APIs only remove cached view/allocation handles, without
   rewriting nominal format or usage. Missing views are typed errors.
2. ResourceDescriptor::from_resource obtains exactly Texture2d/3d::format(),
   preserving nominal channel kind and encoding. Advect receives GpuResource
   whole textures, not TextureView alternatives. Descriptor input generation
   uses sampled texture_2d/3d and its actual scalar base type.
3. AdvectRecipe includes velocity/quantity/output descriptors and BoundaryMode;
   ComputeSourceCache further keys actual device, validation policy and source.
   Changing definition.mode changes the recipe key. Removing the manual flag
   needs no additional cache authority; complete descriptors remain legitimate
   program inputs even where format differences generate identical code.
4. Peer Naga image reflection initializes sampled TextureBinding.format=None,
   retaining the exact image dimension/sample kind. PassParameter selection
   consequently requests each whole texture's own nominal format. The
   same-format path returns its cached view and actual device/dimension.
   Since constructor sRGB excludes storage, effective_format(nominal) equals
   nominal here. The storage-to-linear alternate rule is not reached for
   these sampled Advect inputs. Selection validation and assembly retain the
   selected view and verify device/dimension; no mutation is involved.
5. Output WGSL is an exact storage spelling of the descriptor's linear
   counterpart. A linear float/unorm output stores the shader's linear values
   under that format's conversion policy. A nominal sRGB whole texture is not
   an admitted storage output on the inspected peer path: its cached sRGB view
   conflicts with the reflected linear storage format and its allocation lacks
   STORAGE_BINDING. No new output encoding or silent reinterpretation is
   justified by the input decode repair.

## Separate source issues and disposition

A concrete pre-existing 3D Clamp boundary issue is visible at Advect line 182:
clamp(pos, 0, 1) is multiplied by quantity dimensions then converted to u32.
When a component reaches/exceeds 1, it produces index == dimension. For example,
a size-2 axis, output coordinate 1, dt=1 and velocity component -1 give normalized
position .75 - (-1/2) = 1.25, clamped to 1 then mapped to invalid index 2.
The 2D helper instead clamps integer addresses to dimension - 1. This violates
an algorithmic last-texel clamp and delegates to native out-of-bounds load
policy (Naga supports restrict or read-zero behavior). It is a static source
counterexample, not a measured runtime zero. It is separate from selected-format
repair and must be explicitly inventoried/dispositioned by Root before any
arithmetic change; this investigation proposes no speculative rewrite.

The broad public descriptors can also represent mixed 2D/3D or integer inputs,
while current shader generation chooses dimension from velocity alone and the
2D helper is texture_2d<f32>. Such combinations are not established as supported:
Naga/native construction rejects incompatible generated types. Record that
admission boundary in the full parameter-family audit rather than claiming all
ResourceDescriptor combinations are executable. The unused public
AdvectSignature.dimension u32 is not the cache or execution dimension owner.
No new shape/type contract is needed to remove the erroneous color branch.

## Thirteen-lens source support for the repair inventory

| Lens | Contract and disposition |
| --- | --- |
| Producer propagation | Actual constructor -> format -> descriptor -> sampled binding -> shader linear value is continuous; remove local second decoder, do not insert primitive adapters. |
| Existing type owners | TextureFormat/ResourceDescriptor and canonical selected-view owner already carry format; consume their semantics. |
| Admission contracts | Whole resources, actual device/dimension and native format/usage admission remain authoritative; mixed descriptor combinations are not blanket-supported. |
| Invariant operations | Do not manually decode values already converted by the native load; cache only successful actual builds. Separate 3D clamp counterexample needs its own disposition. |
| Aggregate states | Quantity's encoding cannot own velocity's decoding; the global helper flag couples independent inputs incorrectly. |
| Type ownership | Native view format owns transfer interpretation; Root owns Advect-local recipe source, peer owns texture selection. |
| Redundant authority | IS_QUANTITY_SRGB duplicates a native view conversion and should be removed rather than expanded to two flags. |
| Behavior preservation | Preserve linear-input arithmetic, interpolation, 3D direct loads, ABI and workgroup policy; intended 2D sRGB behavior changes once-decoded samples and untouched float velocity. |
| Enum shapes | BoundaryMode remains its existing closed policy; no new encoding Boolean or fabricated alternate-view enum is required. |
| Derivability | No new serialization/traits/error boilerplate needed; cache/recipe existing derives remain applicable. |
| Type granularity | Keep dimensionality, scalar kind, texture format and transfer semantics distinct; do not treat unused raw signature dimension as admission proof. |
| Magic strings | WGSL names/entry/texture spellings are actual protocol tokens; remove the generated decoder constant and inaccurate comment. |
| Magic numbers | Transfer coefficients belong to the erroneous duplicate decoder and disappear with it; workgroup and interpolation numbers are not part of this color repair. |

## SHA-256 source anchors

```text
4b94a0b3a1a1e51b108608d28ca28fda6178e366ab2c8f49980a4968e2aff721  Root/crates/fabelgeist-compute/src/advect/mod.rs
7e21b07871659c611dbd1aa3db5fbcc4323a55adc4183f8aa3d1d4ba4cac84c3  Root/crates/fabelgeist-gpu/src/data/gpu/compute/mod.rs
686fa8c6510d57714063bb26f287a16c170e690ea25695abd94078041e1d9836  Root/crates/fabelgeist-gpu/src/data/gpu/compute/source_cache.rs
5c67a62164329d7fc2a0d7b96fb4eac6e7cc7317bb6c122ba671ddc95c40335b  Root/crates/fabelgeist-gpu/src/data/gpu/texture/format/storage.rs
60b53619b5d5590c6ebbeb15ca547b2ee4c026c092bc4ac7e9938dec6f674077  d215/crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs
0d8100e52a87510bab94ff9d3cad2d5428152ab3e3d74372376a82df3275881a  d215/crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/views.rs
0ff0d8b9fd6f211ea8a6201e62fbf88efbbe2ca000c60c7f31a1a9b3d12c9f2b  d215/crates/fabelgeist-gpu/src/data/gpu/texture/texture_3d/mod.rs
d65e7afd1fd3c33a56a8df0661712768e562d6665e7e61eea7f61cd9ccdf4fee  d215/crates/fabelgeist-gpu/src/data/gpu/texture/texture_3d/views.rs
8a600e85d0724eb9f9cfb46e9c6d781eec1186c336335f266c7cff2a7e5cc661  d215/crates/fabelgeist-gpu/src/data/gpu/texture/selection.rs
3130b402b58ceb84fa372a2d3b4108517795df791998c26dded8b5bed8d5d145  d215/crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection/images.rs
40d0f30f63f87aebf48b6bd354c40e14c66ff51410ee7586e3d2e59a98dec99c  d215/crates/fabelgeist-gpu/src/data/gpu/bind_group/assembly.rs
0a8f0cf2506ae25d2e185e9dbc22edfe292ef82ad392f58b99a7e82ede45e5da  SDK/wgpu-types-29.0.4/src/texture/format.rs
237289e3a9403a194482559d9f361c7758e35c61bc916283c2fa7d79081b15fc  SDK/wgpu-hal-29.0.4/src/vulkan/conv.rs
a356017af35132873944f45183de968f3ccb605d9b953365488d2cb80ae0cfba  SDK/naga-29.0.4/src/proc/index.rs
```

## Confirmed 3D Clamp addendum

The actual canonical owner `fabelgeist-compute/src/boundary.rs:6-8` documents
Clamp as using the nearest sample on the grid's edge. This makes the Advect
line-182 counterexample separately actionable [P2], not merely a proposed
strengthening. Every successfully initialized peer Texture3d has positive
extents: new() applies at least one per axis; Default has no native view and
fails selected-view admission. Thus valid quantity dimensions minus one cannot
underflow.

The proposed minimal canonical correction is a final integer address guard:
`min(vec3<u32>(clamp(pos, vec3<f32>(0.0), vec3<f32>(1.0)) *
vec3<f32>(dimensions)), dimensions - vec3<u32>(1))`, where dimensions is the
actual quantity textureDimensions. It preserves the existing 3D nearest
sampling, coordinate conversion and interior integer coordinates; it changes
the invalid upper endpoint to the actual last texel. No filtering, alternate
frame/shape policy or general arithmetic rewrite is needed. This is a semantic
boundary repair distinct from removal of 2D's extra color decode and must be
inventoried as such before edits.

Pinned Naga 29 index.rs:10-29 and 39-57 explicitly describe safe texture OOB
redirection, including ReadZeroSkipWrite (native wgpu's documented default),
and Restrict is not promised to mean nearest-edge clamp. Pinned Vulkan adapter
selects Unchecked with native robust image access, otherwise Restrict; Metal
selects Restrict when runtime checks are enabled. Desktop GLES 4.3+ selects
ReadZeroSkipWrite, while other GLES paths use native behavior.
Native safety therefore cannot substitute for BoundaryMode::Clamp semantics.
No observed GPU run or newly added case is claimed.

Additional source anchors:

```text
31cd64fb15c7bb2edcbc355da4483470f1f28ae2e7c061790aa97341abf137b8  crates/fabelgeist-compute/src/boundary.rs
9f9f3557178ac555122d17a472d7cc4289bbde961d2eecf8099ca1b3bd10c24a  /home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-29.0.4/src/vulkan/adapter.rs
c34d2bd9fe2a547f5cb28106c07ed22c99aa5891c0ad6a75f08ec6d70a1f6864  /home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-29.0.4/src/metal/device.rs
7ad87896ba806fcce14ea63d8900e3262b3ac36d3861c3569de9f8576e3b76d9  /home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-29.0.4/src/gles/device.rs
```
