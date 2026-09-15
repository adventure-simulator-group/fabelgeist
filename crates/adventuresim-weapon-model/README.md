# Parametric weapons and fitted holders

This crate owns the precise recipe schema, material catalog, construction,
attachment frames, and physical properties used by the browser weapon modeler,
its CLI exporter, the strategic forge, and tactical equipment. The thin
`adventuresim-weapon-model-browser` crate exposes the same operations through
WebAssembly; there is no second geometry generator in JavaScript.

## Recipes and catalogs

`catalog/authoring.json` contains 42 authoring presets and 20 haft/head
compositions, including ranged weapons, ammunition, carriers, and shields.
`catalog/gameplay.json` contains 23 gameplay chassis. `default_design` consults
only the gameplay registry: it also identifies items requiring an authenticated
per-instance recipe. Broad authoring availability does not change inventory
requirements.

Lengths are precise metres, angles have explicit degree or radian types, and
sampling counts are integers. Components select typed manufacturing shapes and
materials. Their IDs address named frames; display labels do not choose
geometry or animation behavior. Attachments and rotations resolve once in the
kernel. An explicit `opposedTo` dependency links directional heads without
coupling unrelated components. Mechanical animation channels and pivots travel
with generated parts into the GLB exporter.

Durable weapon and holder definitions use versioned JSON envelopes with bounded
size and strict field decoding. Hashes cover canonical design serialization,
domain separation, and schema/generator versions. Recipes validate
before encoding, generation, or physical derivation. Consumers authenticate
bytes, chassis identity, generator version, and hash before using cached output.
No compatibility decoder is provided. Recipes are strategic equipment data;
tactical positions, damage, and combat state remain transient.

## Physical construction

`generate_model`, `generate`, and `derive_properties` use the same canonical
construction. Signed tetrahedral integration gives material mass, center of
mass, and mean transverse inertia about the controlling grip. Geometry gives
length and reach. Physical properties use fixed High construction accuracy;
Low and Medium display meshes cannot change authoritative handling or material
requirements. Material contributions retain their typed identity, and their
sum conserves total mass.

Physical evaluation uses the material solids directly; renderer normals, colors,
and vertex indexing are built only for display. `EvaluatedWeapon` and
`EvaluatedHolder` own an immutable validated design and its derived properties.
Consumers can reuse one evaluation to authenticate the canonical recipe, hash,
and stored projections, then read its physical values without rebuilding the
construction. Ownership and inventory identity checks remain at the consumer.

Successful physical evaluations retain at most 64 complete design keys per
weapon or holder cache. An edited design uses the same authoritative
construction on a cache miss; failed evaluations are never retained. Cache
hits do not bypass transport limits, version checks, or authentication of
canonical bytes, hashes, and stored physical projections.

Blade thickness is maximum forte depth. Width and distal taper, curvature,
section, materials, and attachments affect both geometry and physics. Shape
choices distinguish meaningful constructions, such as a flat or diamond spear
section and a forged plate or round-tube figure-eight guard.

Fitted sheaths have material walls around a blade-clearance envelope, an open
throat, and a closed distal cap. Metal fittings share the body's supporting
planes without counting its material twice. Haft loops include bar radius in
their free-clearance construction. Holder mass comes from these material
solids and is used by strategic and tactical carried-weight calculations.
Assembled overlaps and unmodeled fasteners remain limitations of estimated
mass; generic authoring ranges are not historical certification.

## Development

Run `cargo test -p adventuresim-weapon-model` for schema, geometry, physical,
editor, icon, and holder checks. From `tools/weapon-modeler`, `npm test` builds
the kernel and runs browser boundary and independent geometric checks;
`npm run test:quality` runs the larger quality corpus. Standard start and export
commands also build the kernel. See the modeler README for prerequisites and
reference-capture commands.

`node tools/weapon-modeler/check-kernel-parity.mjs` compares native and WASM
request output. The `audit_catalog` example exports exact gameplay definitions,
meshes, and measurements; the browser catalog audit uses the same kernel for
authoring entries. Review transcripts belong under ignored `target/` or
`output/` directories.

## Equipment portraits

Weapon and holder icons use an orthographic color renderer with a depth buffer,
interpolated surface normals, material colors, and a studio environment light
map. Guard and head frames use the modeled anatomy; synthetic staff head framing
stays independent of shaft tessellation. Outputs are opaque RGBA images on black
squares; consumers display their colors directly.
`ICON_RENDERER_VERSION` invalidates the transient recipe caches when rendering
changes. Export a portrait with:

```sh
cargo run -p adventuresim-weapon-model --example export_icon -- longsword target/longsword.png
```

Armor portraits use the finished, fitted GLBs, including their material and
normal textures. `just equipment-icons assets/equipment/procedural` exports the
shared studio HDR and invokes Blender 5.2 through `BLENDER_BIN`. It writes each
placement's portrait beside the tactical meshes in `icons/` and into the
strategic web static assets. The equipment generation workflow runs this after
finishing the meshes. Rebake portraits after changing geometry or finishes.
Tactical equipment uses its placement variant; strategic rows use the first
placement in manifest order sorted by placement ID.

## Combat contact

The shared core consumes these physical recipes to derive continuous contact
precision. Handling accuracy comes from grip-to-tip length and imbalance. The
contact model, calibration anchors and limits are documented in
[the contact review](../../tools/weapon-modeler/review/1544-audit/contact-model.md);
the weapon schema contains no accuracy bonus or damage-type flags.
