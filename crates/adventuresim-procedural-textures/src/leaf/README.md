# Shared procedural leaves

All leaf recipe IDs use one CPU field evaluator on native and WASM. Species
select `LeafParameters` data from `TextureParameters.leaves`; they never select
a shape algorithm. The 38 shape presets include the reference catalog plus
hazel, blackthorn and hawthorn. Living and dry oak share a shape with different
palette and relief data.

The structural equations and coupled constraints are adapted from
[adventure-simulator-group/leaves at 17b60b0](https://github.com/adventure-simulator-group/leaves/tree/17b60b0a6f817babd1ce27467645d9ecb8389277).
`shape.rs` names the 68 independent scalar controls. `uniform.rs` packs them for
`constraints.rs`; the focused profile, organ and venation modules evaluate the
same fields as the reference WGSL. Two reference CLI convenience macros are
represented by their independent underlying controls, rather than duplicated
state. Preset values are clamped to the reference's public scalar ranges.

`LeafShape::interpolate` blends the actual structural controls and preserves
exact
endpoints. Counts remain floating point and progressively grow entering organs.
The reference's architecture precedence and raster constraints still apply:
incompatible families can suppress controls, so arbitrary cross-family morphs
are not promised to be smooth everywhere. Constraints act at bake time without
silently rewriting the artist's document.

Albedo contains blade and vein colors, with intermediate colors only where a
2-by-2 sample footprint crosses a class boundary. Coverage and color mips retain
antialiasing. Height and normal detail are separate from pigment. A filtered
vein mask drives rounded vein relief; dome, curl, corrugation and tissue
controls add surface relief. Both normal maps derive from their own height
field, reversing the back map's green channel for the back-face tangent frame.

The pinned WGSL in `fixtures/reference.wgsl` is an independent test oracle. To
compare CPU samples against WebGPU (with Texture Studio served on port 8783):

```powershell
$env:FABELGEIST_LEAF_PARITY_OUTPUT = "$PWD/target/leaf-reference/parity.json"
cargo test -p adventuresim-procedural-textures export_reference_vectors -- --ignored
node scripts/test_leaf_reference.cjs target/leaf-reference/parity.json
```

The Node runner requires Playwright and WebGPU-capable Chromium. It tests all
presets and an oak-to-beech morph sweep, allowing at most one raster-boundary
sample difference per 1,024-sample case for CPU/GPU floating-point rounding.
