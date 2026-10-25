# Independent Code Review: Advect decode and clamp delta

No actionable defect found in these two repairs. This verdict covers only the
identified decode and 3D Clamp defects. The broader generated-program family,
AST/ordered-role/private/default/item-order and launch producer gaps remain
OPEN; the entire Advect module and complete PR are not cleared.

Root HEAD `fff01705f4eaae22f98242ce64e538e02ac76e86` plus one working source change. Current Advect SHA-256 is
`55748934ab8638b33ef380aae35e12c0ab61b888158a5e0b5de459c68495ba7a`.
Read full current Advect and its diff versus HEAD; the ignored original source
is byte-identical to HEAD. Read the discovery report and complete two-defect
all-thirteen preinventory. Rechecked actual Root constructor/selected-view/image
reflection contracts and pinned wgpu-types 29.0.4 transfer semantics.

The 2D helper now returns interpolated native-load values directly. Removing
both the Rust quantity classifier/emitted constant and WGSL branch eliminates
the duplicated sRGB transfer and the same incorrect transfer on float velocity.
The same sampled-view contract holds in current Root: constructors create the
nominal cached view, sRGB omits storage usage, and reflected sampled bindings
have no format override. Native SDK loads already provide linear shader
values for sRGB and do not convert float formats. No alternate format getter,
view/cache metadata mutation or replacement application decoder was added.

The 3D Clamp path queries actual quantity dimensions, performs the prior
normalized clamp/float multiplication/u32 conversion unchanged, then takes the
component-wise minimum with dimensions - vec3<u32>(1). For admitted initialized
positive extents, subtraction is safe. For interior addresses the integer
minimum leaves the old address unchanged; at an upper endpoint it selects the
last valid texel. A size-2 axis with advected normalized position 1.25 now maps
to index 1, as BoundaryMode::Clamp's nearest-edge contract requires, rather than
invalid index 2. This is source reasoning, not an executed case.

WGSL shape/type reasoning: sampled texture_3d textureDimensions yields
vec3<u32>; vec3<f32> conversion, component-wise multiplication, vec3<u32>
conversion and min operands are type-consistent. The abstract literal in
vec3<u32>(1) is representable. The following textureLoad continues using the
same sampled quantity, vec3<u32> coordinates and mip zero, and textureStore is
unchanged. Added locals have distinct names and are declared before use in the
3D Clamp entry only. This review did not execute a shader parser/compiler.

Independent source reversal reinstated exactly the classifier block, manual
transfer block and former single Clamp load, then compared the reconstructed
whole file with the baseline: True. Thus all other source—including interpolation, Wrap, 3D sampling,
cache recipe, execution parameters, bindings, uniform ABI and dispatch
policy—remains byte-identical. The parent comparison JSON records matching
before/after hashes as supplemental evidence.

No Advect tests.rs or inline test module exists. No borrowed fixture was
invented. No tests/cases/helpers/assertions, source edits, local project checks,
CI requests or external actions were performed. Final complete waves,
independent combined reviews and hosted project checks remain pending.

## Fingerprints

```text
55748934ab8638b33ef380aae35e12c0ab61b888158a5e0b5de459c68495ba7a  crates/fabelgeist-compute/src/advect/mod.rs
4b94a0b3a1a1e51b108608d28ca28fda6178e366ab2c8f49980a4968e2aff721  target/pr780-audit/advect-decode-clamp-before.rs
31cd64fb15c7bb2edcbc355da4483470f1f28ae2e7c061790aa97341abf137b8  crates/fabelgeist-compute/src/boundary.rs
537cbf39f00bc36c4f46ff60a6b550037b283eedafcf41d3975274e5f3e61184  crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs
31fcc315c4314f2d63153aac4b2db696eae5a02d5ce7ae97881460e06a251295  crates/fabelgeist-gpu/src/data/gpu/texture/texture_3d/mod.rs
8a600e85d0724eb9f9cfb46e9c6d781eec1186c336335f266c7cff2a7e5cc661  crates/fabelgeist-gpu/src/data/gpu/texture/selection.rs
2a09fff5d2d38ccd60190e1d41cd806f8298d3c40cb27bccb2a7844febdc7952  crates/fabelgeist-gpu/src/data/gpu/texture/texture_3d/views.rs
3130b402b58ceb84fa372a2d3b4108517795df791998c26dded8b5bed8d5d145  crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection/images.rs
ed1f1f795d6aa1f88296643baaa2cad4c5fe7bf175a70c9555801ab708ad86da  /tmp/pr780-advect-selected-format-discovery.md
```
