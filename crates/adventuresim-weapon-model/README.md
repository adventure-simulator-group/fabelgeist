# Parametric melee weapons

Recipes own component dimensions, materials, attachment frames, and
manufacturing shapes. `derive_properties` and `generate` construct the same
unshaded solids. Signed tetrahedral integration gives material mass, center of
mass, and mean transverse inertia about the controlling grip; geometry bounds
give total length, head extent, and grip-to-tip reach. Physical properties do
not use catalog mass, balance, or reach corrections. Derived mass by material
conserves total mass.

Blade thickness is the maximum forte depth for flat, diamond and fullered
sections. Width taper, distal taper, curvature, material and attachments affect
the same geometry used by the physical calculation. No shading buffers or
rendering normals are constructed when only physical properties are requested.

Generator version 8 identifies this geometry and property contract. Durable
recipes remain strategic equipment definitions; no tactical state is persisted.

The
[1544 audit](../../tools/weapon-modeler/review/1544-audit/historical-assessment.md)
provides references and scope limitations. Mass is a component construction
estimate: component overlaps and unmodeled tangs/fasteners remain explicit
limitations. The browser authoring modeler is a separate generator and is
reviewed separately. Generic authoring ranges are not historical constraints.

Run `cargo test -p adventuresim-weapon-model` for geometry, physical-invariant,
editor, icon, and historical-envelope regressions. The `audit_catalog` example
exports every preset and every gameplay catalog recipe as exact definitions,
triangle meshes, and physical measurements for independent review.

## Equipment portraits

Weapon and holder icons use an orthographic color renderer with a depth buffer,
interpolated surface normals, material colors, and a studio environment light
map. The existing guard and head framing remains unchanged. Outputs are opaque
RGBA images on black squares; consumers display their colors directly.
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
