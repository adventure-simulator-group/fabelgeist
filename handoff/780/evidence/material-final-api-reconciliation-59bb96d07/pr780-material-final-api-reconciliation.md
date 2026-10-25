# Material final API reconciliation before implementation

Design/discovery only at HEAD `59bb96d07af08244582c6767d81e969b820ec3ed`.
The completed density/Armory eight-file review remains preserved. No source
edits, project execution, tests, probes, fixture helpers or assertions were
performed. Current source APIs were inspected; the merged 91-account ledger
and peer reports were maps, not independent clearance of unread source.
Partial source extents and current fingerprints are in the companion JSON.

## Minimum final owner and API

The proposed `SurfaceMaterial` fits the actual contracts. Define its sole
actual implementation in focused Armor `surface_material.rs`; move/replace
the existing `ArmorComponentMaterial` definition, remove its old public export,
and directly reexport the actual new owner. No type alias, old forwarding API,
Creator PBR type or raw tuple adapter is needed.

The minimum value is exactly encoded float RGBA, metallic and perceptual
roughness:

```rust
pub struct SurfaceMaterial {
    pub base_color: fabelgeist_color::Srgba,
    pub metallic: fabelgeist_math::units::Ratio,
    pub roughness: fabelgeist_math::units::Ratio,
}
```

Clone/Copy/Debug/PartialEq and serde preserve current scalar-record behavior.
No universal Default is warranted: inherited appearance is not a white surface.
A composite constructor, if supplied, takes these typed leaves, not three
primitive parameters; struct literals with typed fields are sufficient.
Ordinary literal/catalog boundaries may construct the unit-only leaves.
No global [0,1] or Metal .08..=.9 bound belongs here. Current export validates
input finiteness only and admits finite out-of-unit values. Wool/TextileColor
.92 roughness, mail up to one and existing fixture one remain accepted.
Unit-only Ratio and encoded color do not claim finite admission.

`fabelgeist_color::Srgba` means an actual new Color-crate type/reexport, not
`Color::Srgba` as an enum variant: the current Color is already a public
four-component struct. Keep it intact for its existing consumers. Encoded
Srgb/Srgba should use private fixed-array storage, explicit unit/encoding-named
literal/native constructors and readonly native projections, transparent array
serde, and canonical transfer functions. Preserve alpha directly; decode only
RGB. Never quantize float material factors through TextileColor or u8 palettes.
TextileColor itself remains its authored u8 owner, with its exact f32 /255
producer conversion into the shared float PBR value.

The minimum component schema is one required field:

```rust
pub enum ComponentAppearance {
    Inherit,
    Surface(SurfaceMaterial),
    Textile(TextileAppearance),
}
// ArmorComponent: pub appearance: ComponentAppearance
```

Clone/Debug/PartialEq/serde fit; Copy does not, because TextileAppearance owns
vectors/boxes. A default enum variant may express actual inherited state for
Rust construction, but do not add a serde default that silently accepts missing
appearance. Retain the explicit absence state; it is not a default material.
The actual owner may provide `textile(&self) -> Option<&TextileAppearance>`,
`textile_mut(&mut self) -> Option<&mut TextileAppearance>`, and
`material(&self) -> Option<SurfaceMaterial>` as direct payload/derived-value
queries. Inherit yields None, Surface copies the one value, Textile derives
from the actual recipe. These are intrinsic enum operations, not legacy shims.
Direct exhaustive matching is also sufficient; no extra facade is required.
Do not cache a second derived PBR beside a mutable textile payload.

Delete local ClosureMaterial rather than rename its dual fields. Fastener
construction/component publication can carry `ComponentAppearance` directly;
Surface handles hook/metal-eyelet PBR and Textile handles button loops, sewn
rings, cords and sewn_eyelets. Clone that one canonical payload when each
component is published. The cord's fabric/wire section-role query can inspect
the canonical variant without extracting and readmitting a scalar.

## Exact naked-triple producers and retained API propagation

| Current owner/API | Direct final API |
| --- | --- |
| `clothing_material::pbr(EquipmentMaterial) -> ([f32;4],f32,f32)`; publicly reexported as `equipment_pbr` | Return SurfaceMaterial directly; keep ten authored entries and PolishedSteel's Metal default source. |
| `garment_material::CLOTH_PBR` tuple constant | Typed SurfaceMaterial constant with the same [.52,.42,.28,1], 0, .85. |
| `MailWeave::pbr()` tuple | SurfaceMaterial with white encoded factor, metallic one and actual weave roughness; color still lives in maps. |
| `MailSurface::pbr()` tuple forwarding the retained weave | SurfaceMaterial from the same weave; this meaningful export-ready owner need not be removed. |

`GarmentSpecification.material` already means EquipmentMaterial; retain that
selection and replace its three raw fields with `appearance: SurfaceMaterial`.
The constructor, whole-specification clone into ClothingShell, preview and
rigged_clothing must carry this one record without defaults/reconstruction.

`RiggedShell.surface` already means optional plate UV plus ShellTextures.
Do not collide with it by naming the new PBR field surface. Use
`material: SurfaceMaterial` (or consistently appearance) and remove the three
raw fields. Copy the whole record in CompactShell::rigged, catalog plate
replacement, component replacement, trim/lacing construction, mail/cloth
DrapedGarment::rigged and hair shell construction. Keep texture/finish/alpha/
side/sampling metadata as their distinct owners.

`armor_metal::metal` currently destructures catalog PBR into color/roughness:
consume the canonical aggregate/encoded-color projection directly at the Metal
recipe construction, retaining its actual Metal policy validation. The two
catalog export paths assign one aggregate instead of three independently
mutable fields. Trim uses its selected Metal source and metallic one; lacing
uses its selected cord source and metallic zero. The fallback in rigged_armor
should consume the canonical Metal-default appearance producer rather than
repeat literals that happen to match it. Hair remains white, nonmetallic,
HAIR_ROUGHNESS with its existing coverage-mask textures; no metal default.

The exact existing fixture helper `export_equipment_with_maps` currently takes
three primitive appearance arguments. Change that existing signature and its
existing callers to SurfaceMaterial; do not keep a tuple shim or add a helper.
Existing MailSurface fixture's `.pbr().2` becomes the canonical roughness field
projection without changing its oracle or scenario.

## Every component receiver must consume the one authority

`device_textile_materials::apply` writes Textile(fabric.clone()) once instead
of material+textile separately. Its skirt support branch writes Surface from
the support color, clearing the possibility of a stale previous textile.
`device_textile_relief::apply` mutates relief/needlework chart data and installs
pleat payloads through the real textile variant. This is actual geometry input,
not a disposable shading cache. `split_textile_seams` must select the same
Textile variant and preserve cloned appearance through exact range/vertex
remapping. DevicePiece component partitioning clones appearance with the rest
of each component and preserves the one canonical payload at endpoints.

Studio preview currently gives textile precedence over the independent PBR;
export currently first copies PBR and then only adds textile finish/textures.
The final enum fixes that concrete divergence: Inherit preserves the catalog
surface/maps; Surface replaces PBR and clears inherited texture/plate bindings;
Textile replaces PBR from that exact textile, clears inherited plate bindings,
and installs its finish and its optional maps. Do not clear only the PBR while
leaving inherited metal maps attached to a textile, or only add textile maps
while retaining the old metallic/roughness. Scalar opacity remains when no
color map exists. Color maps replace factor with white, so they are not tinted
twice. Current textile terminal validation remains in textile_surface helpers.

Tactical MaterialPartition currently compares both independent fields and
stores a borrowed textile. Store/compare one borrowed canonical appearance.
Normalize explicit Inherit and untagged faces to the same inherited group so
existing draw grouping remains equivalent. SurfaceMaterial stays Copy and
PartialEq; the whole ComponentAppearance is borrowed/Clone, not Copy.
Textile rendering delegates to the same textile_surface owner; Surface projects
Srgba and Ratio only at the actual Bevy constructor; Inherit uses the catalog
SurfaceMaterial producer. Do not compare only derived scalar PBR for textiles:
different recipes with equal color/roughness can have different maps.

## External schema and dependency implications

The old ArmorComponent derives public Serialize/Deserialize, so replacing its
independent material/textile fields is an externally visible schema redesign,
even though current GeneratedArmor/BuiltPart retained structs inspected here
do not themselves derive serde. Select and record the final representation
before edits. A clean snake_case externally tagged enum is sufficient, e.g.
appearance `"inherit"`, `{"surface":{...}}`, `{"textile":{...}}`.
Do not add old-name aliases, untagged dual decoders, legacy fields or
migrations.
Do not serde-default a missing appearance and then silently discard old fields.
A required appearance plus explicit unknown-field rejection is the clean schema
if malformed/obsolete records must fail rather than lose their appearance.
This is a deliberate rejection/wire-shape change; no public stored component
payload inventory beyond the inspected receivers is claimed complete here.

SurfaceMaterial fields should preserve the prior RGBA array and scalar wire
representations. Reusing existing generic Color's named r/g/b/a serde would
change that shape; transparent Srgba avoids it. Component enum schema is
intentionally new; generic material scalar arrays need not change. The old
record has no Default; neither a derived scalar default nor an inherited white
material should be invented.

Armor needs a direct Color dependency with serde enabled for the new leaf;
it already has Math, serde and thiserror. Creator also needs a direct Color
dependency for actual new encoded leaf/transfer use. Tactical already depends
on Armor/Math but needs direct Color dependency if its fixtures or production
construct/import Srgba themselves; Rust transitive dependencies are not direct
API access. Normal receiving getters alone can avoid an unnecessary direct
Tactical Color import, but fixture literals still need a concrete owner.
Color's wgpu feature must remain unnecessary; avoid introducing a dependency
cycle or renderer dependency into Armor/Color. Reconcile both actual lockfiles
and the chosen Color serde feature composition. No new high-level crate or
runtime framework is needed.

## Additional source-backed native projection hazard (P2)

`export/validation.rs:154-156` accepts any finite encoded RGBA; current
`export.rs:578-588` converts encoded positive RGB with f32 powf(2.4), and
`export/shell.rs` places the result in JSON. A finite encoded channel 1e20
passes admission but its decoded value is approximately 1e48, beyond f32
representability. Actual pinned serde_json **1.0.151** serializes a nonfinite
f32 Value as Null (`value/from.rs:30-44`, `value/ser.rs:151-153`). Thus the
current public finite-input path can emit a nonnumeric color factor. This is
source-derived arithmetic evidence, not an executed test or assertion.

Do not fix it with [0,1] bounds or clamping. Preserve generic out-of-unit input
semantics while separately admitting actual native output representability:
perform canonical decode and a finite output check before JSON/GLB publication,
propagating a real error for unrepresentable projection. If keeping exactly
the old f32 formula, this is a deliberate earlier error for a broken public
output path. Widening the color conversion is a different numerical contract
and must be inventoried explicitly. Shared Srgba itself should not pretend
all finite encoded values have finite f32 decoded values. Root must add this
receiving finding to all thirteen preinventory cells before its source repair.
No new fixture/case/helper/assertion is authorized.

Two already inventoried receiving repairs remain separate: body skin writes
encoded factors without decoding, and CompactShell admits plate-only UVs but
copies their stale source stream after vertex remapping. Canonical PBR/Color
API propagation cannot silently declare either fixed; preserve their named
preinventory and connected output acceptance.

## Existing fixture and behavior acceptance

Update only necessary type/API syntax in existing ArmorComponent/RiggedShell/
GarmentSpecification constructors, existing helper parameters and tuple field
accessors. That includes tactical's existing red/gold component values,
textile_export's existing fabric-derived material, export inline literals,
assembly/morphs/texture fixtures, existing garment pbr roughness oracle and
existing skin/component constructors. Do not add cases/helpers/assertions or
weaken/rewrite their numerical/topology/PNG/alpha expectations.

Keep exactly: u8 palette division, arbitrary float sRGB precision, alpha,
finish-dependent metallic/roughness including .92, texture white-factor
replacement only when actual color bytes are present, normal/AO linear format,
textile specularity, Mask/Blend/Opaque, side and sampler policy, effective UV
precedence/association, and catalog/trim/cord/mail/hair source selection.
The float RGB egui repair must use canonical decode -> local linear picker ->
changed-response guard -> canonical encode; never commit an unchanged frame's
round trip or force float recipes through bytes.

## Thirteen design dispositions and remaining conditions

1. Producer propagation: the four exact tuple owners, garment/shell retention,
   all component constructors and native receiving graph must migrate directly.
2. Existing owners: promote Armor's existing PBR concept once; Color owns
encoded
   leaves/transfer and Math owns Ratio. TextileColor remains separate u8 data.
3. Admission: preserve unit-only leaves and terminal finite/recipe checks;
   explicitly reject unrepresentable native color output, not generic ranges.
4. Operations: component replacement is one payload assignment; alpha/finish
   updates use typed owner fields. UV remapping and color projection need their
   own connected output admission, not rewrapped primitives.
5. Aggregate states: required Inherit/Surface/Textile closes dual authority;
   textures/UV/finish state must be replaced together in actual receivers.
6. Ownership: focused Armor module plus one real public export; no Creator/Bevy
   owner or old-name alias. Preserve separate maps and physical ShellMaterial.
7. Redundant authority: no derived PBR next to fabric, no old raw garment/shell
   columns, no second sRGB formula, no stale plate-only UV fallback authority.
8. Behavior: retain exact scalar data and valid absence; schema and the named
   output corrections are deliberate changes, not mechanical preservation.
9. Enum shape: one enum owns appearance payloads; keep mutable textile access
   for real relief/seams; no semantic bool or two-option shadow record.
10. Derivability: preserve appropriate serde/Clone/Copy/PartialEq on value
types;
    enum is not Copy, no new manual-trait/default boilerplate is needed.
11. Granularity: shared SurfaceMaterial has typed leaves but no Metal-specific
    ranges; component appearance, maps, geometry and authored TextileColor stay
    distinct; no per-consumer material wrappers or raw triple adapter.
12. Strings: final enum wire spelling is closed and explicit; real names/paths/
    glTF keys remain external or presentation protocol, never error classifiers.
13. Numbers: authored catalog coefficients remain literal boundaries with typed
    leaves; canonical transfer/density owns shared conversions; output finite
    arithmetic is checked without new tolerances or magic range clamps.

This reconciles the concrete API design and identifies required source/schema
choices; it is not implementation or final Rule clearance. Root's complete
merged ledger, actual final source review, same-revision independent reviews
and hosted gates remain required. Unread durable component payload users,
full native map/rasterizer math, complete exterior API clients, broader physical
families and the separate generated tranche remain unverified here.
