# PR #780 material semantic family: thirteen-lens preinventory

This is a read-only discovery before material type repairs. All thirteen lenses
were applied sequentially in the order below. The pinned current source account
is `/tmp/pr780-material-semantic-all13-discovery.json`: HEAD
`f2c7422fd6a0f995eb90b72cc9c2a5bd12de4d35`, 96 repository source/configuration
accounts and two exact pinned dependency-source accounts, each with SHA-256 and
explicit full/partial coverage. Root advanced commits during inspection; these
fingerprints identify the actual bytes, rather than an earlier verdict.

No source edits, project checks, tests, probes, quality scanners, generators,
new cases/helpers/assertions, hosted CI or external publication were performed.
This report does not close material migration or the PR. Root remains the sole
implementer. Code Review is separately completing the cause/density family;
Comment Review is separately inventorying receiving documentation. Their coverage
must be composed with this inventory, not inferred from this report.

The repository root and crates/AGENTS.md, rule-review and its bespoke reference,
and all thirteen focused audit skills apply. Existing MAT-C1 and MAT-R1 through
MAT-R5 are still open. Newly confirmed color-boundary finding MS-COLOR below is
connected to R1/R2; the other new details refine the connected repair contracts.

## Current field and boundary inventory

| Owner and source | Contract and actual consumers | Before-repair disposition |
| --- | --- | --- |
| Metal.color, material.rs:15-30 | Three artist-facing encoded sRGB f32 channels, finite 0..=1 by Metal::validate. Default [0.769,0.776,0.776]; trim default [1,0.77,0.34]. Preview uses Color::srgb; catalog/export use the same appearance and export converts sRGB factors to linear. No alpha is stored here. | R1: retain a real encoded-color owner from construction/serde through cache and consumers; do not quantize to u8 or silently add a fourth wire channel. MS-COLOR: the actual egui picker currently expects linear input. |
| Metal.roughness, grain, smudge | Dimensionless values; roughness 0.08..=0.9, grain 0..=1, smudge 0..=0.4, all finite. These limits differ from catalog dielectric roughness (e.g. wool 0.92), normal-map strength (up to 4), and rope slant (up to 3). | R1: canonical Ratio is a useful unit leaf; each application record retains its actual bounds. Do not apply Metal's bounds to all PBR values or clamp every ratio to one. |
| Metal.waviness | Metres, finite 0..=0.002; default 0.0004. UI displays millimetres. Native finish multiplies by tiles per metre to derive slope. | Carry canonical Metres through the producer; valid zero rules out PositiveLength as a direct replacement. Keep the existing millimetre division order at the actual UI adapter. |
| Metal.scratch_density | Number of scratches per metal tile, inclusive 0..=3000, default 500. It determines workgroup y and the five-draw-per-scratch stream offset. Zero skips stamping. | Purpose-specific count, not a buffer population, scan count, sort quantity, workgroup count or physical areal density. Carry it until native launch/LCG projection. |
| Metal.scratch_length | Fraction of a metal tile, finite 0.005..=0.4, default 0.03. Native span = scratch_length * tile side * seeded reach. | Dimensionless Ratio/role, not metres. Preserve exact multiplications and rounding. |
| Metal.scratch_width | Texel radius scale, finite 0.5..=3, default 0.65. Native distance in texels is divided by this width. | A fractional texel span, not an integer pixel count, metre value or unit-interval coefficient. Select an exact float-texel owner; do not misuse BufferElementCount. |
| Metal.scratch_depth | Relative height/groove scale, finite 0..=1, default 0.02. It scales native groove height; adjacent height differences and named roughness amplitude consume it. | Dimensionless scale; do not relabel it as physical engraving depth. |
| Metal.scratch_angle/spread; Engraving.rotation | Radians; angle and rotation finite -pi..=pi, spread finite 0..=pi. Native sin/cos and the seeded swing consume the exact represented values. | No portable general Radians owner exists in Math/Geometry. Building generator's finite Radians is coupled to its Bevy/frame module; importing that crate is wrong. A shared portable leaf belongs below Armor/Creator, with each field retaining its sign/range policy and no angle wrapping. |
| Metal.seed | Full u32 word, any bit pattern, default 1544; native wrapping 32-bit LCG uses multiplier 1664525, increment 1013904223, top-24-bit draw. | Meaningful native32 seed owner required. Canonical determinism::Seed is a u64 record with different serde shape and derivation semantics: not a drop-in substitution, truncation/hash adaptation or excuse to replace this generator. |
| Metal::TILES_PER_METRE | Positive spatial frequency 4.0 repeats/metre. Native shader source, trim period, fitted catalog UV rescale, tiling chart and lacing chart producers consume it. | Carry canonical PerMetre through the actual producers/operations. The unchanged tiling/mod.rs and tiling/lacing.rs consumers are part of this migration; projection belongs at their numerical UV kernel or generated WGSL boundary. |
| Engraving.tiles | Fractional repeats per metal tile, finite 0.25..=64, default 1.0; source sampling, trim period/origin and native ornament use it. | Ratio/spatial repetition scale, not integer Ornament.repeats. Do not remove fractional repeats. |
| Relief::Height.depth | Metres, finite 0..=0.005, default ETCH_DEPTH 0.0004. Zero still selects Height and yields an optional zero depth map; it is not Normal or missing relief. | Use Metres and preserve zero/type distinction. MAX_DEPTH is a physical bound; no PositiveLength substitution. |
| Relief::Normal.strength | Finite amplification ratio 0..=4, default on UI transition 1.0. Image slopes and native rotation consume it. | Carry Ratio through source/decoded/binding operations; do not clamp to one or convert to metres. |
| Engraving.recess_roughness | Finite dimensionless additive amplitude 0..=1, default 0.35. Native bake combines it with the actual cut. | Carry Ratio, preserving the amplitude role and same arithmetic. |
| ReliefSource/Relief/Cut | Open PathBuf image source or closed procedural Ornament; images admit Height or Normal, Ornament admits Height only. Engraving::validate currently rejects the representable Ornament+Normal combination. EngravingImage/Cut retain immutable recipe borrows; decoded ReliefPixels is Height or Slopes. | Aggregate/enum refinement: source variants should own only supported relief payloads, or checked aggregate construction must make the correlation explicit. Keep genuine open paths; no enum of filenames. Preserve the real decoded-image fixture path and diagnostic source. |
| Ornament/motif | Repeats 1..=8, line fraction 0.01..=0.2, fillets bool semantic choice. Wave/Zigzag amplitudes 0..=1; Guilloche amplitude 0..=1 plus strands 1..=6; Rope slant 0.1..=3; Beads radius 0.1..=1; Vine amplitude/leaf 0..=1. Defaults and suggestions are authored motif policy. | R1/R2: typed repeat/strand roles and ratios; FilletLines enum or equivalent named carried role, with an explicit bool wire/checkbox adapter. Do not use Hair strand count or TextileThreadCount merely because integers fit. Native motif code/first/second/strands must be a named projection, not a positional application tuple. |
| Trim recipe width | Metres, finite 0.003..=0.06, default 0.02; holds its own Metal. | R1: typed physical leaf and explicit authored bounds. Geometric TrimBand is a separate contract. |
| Trim.period/cell_origin/cell_tiles/narrowed | Period = 1/(4*tiles), metres; origin = 0.5-0.5/tiles, dimensionless and legitimately negative for 0.25 repeats; cell_tiles = clamp(1/(4*width),0.25,64). Narrowing scales engraving repeats, then clamps. Construction's widest_trim floors at recipe MIN_WIDTH. | Carry Metres/PerMetre/Ratio rather than raw application f32. Preserve operation order and clamping. Public narrowed currently admits zero/negative/sub-minimum input and can return an invalid Trim; choose coherent fallible/admitted final API. Existing test narrows 0.02 by 0.25 to 0.005, above MIN_WIDTH: it does not establish sub-minimum support. |
| TrimBand/ArmorTrim/Nearest/Split | Geometry admits finite positive width and period without recipe min/max. Rim distances/along/closure are metres, closure zero means open; nearest uses the same plate membership. ArmorTrim coordinates pair metres along rim and in from rim. Cuts preserve all vertex attributes/morphs/skin; components are regrouped. | Existing PositiveLength matches geometric band leaves; do not impose the recipe's 3..60mm range on this API. Carry physical leaves through edges/split/retained coordinates, or mark the exact real numerical geometry boundary. Named along/inset record prevents swapping semantic coordinates. Keep mesh-index/plate identity issues separately inventoried, not implicitly solved by unit names. |
| PBR records | clothing_material::pbr returns ([f32;4],f32,f32); ArmorComponentMaterial already owns named base_color/metallic/roughness. GarmentSpecification, RiggedShell, equipment preview/export and tactical runtime retain/consume these values. | Reuse/refactor the existing shared appearance owner rather than inventing a competing Creator Pbr struct. Give fields actual color/ratio leaves. ShellMaterial is physical compliance and not equivalent. An owner move/rename must update all users directly, without aliases or old/new dual paths. |
| Trim/depth/native/table tuples | armor_metal::trimmed returns (GeneratedArmor,UVs); armory and selected equipment carry further positional tuples; MetalImages.depth stores (Handle<Image>,uv_scale). Metal validation table has (value,low,high); Motif::words has four roles. | R2: small named records with typed fields, associated constructors when constructing a local type, sole authority. Do not add pass-through helper/adapters that merely reconstruct primitives. Required Bevy bundle tuples and exact mathematical sine/cosine pairs remain boundary/math representations. |
| MetalTextureSize/maps | Supported square side 32..=1024, PREVIEW 1024; derives canonical buffer population and checked word layout. MetalTextures privately owns size+maps and admits exact RGBA8 lengths plus finite nonnegative depth uv_scale. | Keep this existing admission. Depth uv_scale is a dimensionless texture-coordinate depth scale, not metres. Transfer packets/raw pixel bytes are actual native/image ownership boundaries. Preserve optional zero depth and selected shader roles. |
| Decoration/library/cache | Optional engraving and trim are independent; plain empty is valid Decoration but rejected as a stored library entry. LibraryName owns open names; PathBuf is an open native source path. Preview caches Metal recipes while used; cache key includes image path/recipe but no file-content revision. | Keep optional axes independent. Existing documented cache is a baked snapshot while used; do not claim live asset freshness or invent a watcher. If a reload/freshness contract is introduced, inventory image-content dependencies explicitly. UI text-edit buffers are genuine text boundaries, not justification to erase typed source paths in retained recipes. |

## New confirmed color-boundary defect

**MS-COLOR, P2, connected to MAT-R1/R2.**
`metal_controls.rs:12` passes encoded float sRGB Metal.color directly to
`egui::Ui::color_edit_button_rgb`. `construction_controls.rs:73` does the same
for Lacing.color, which `preview::lacing_material` interprets through
`Color::srgb`. The pinned egui 0.34.3 `ui.rs:2274-2280` explicitly specifies
linear RGB. Its actual `widgets/color_picker.rs:548-554` constructs linear Rgba
and writes the linear components back; the receiving renderer/export treats
those same components as encoded sRGB. Thus the picker and rendered/exported
color have different transfer functions for the same authored values.

The repair belongs at the real UI adapter: preserve float encoded storage,
convert to the framework's required linear representation using canonical color
conversion, then commit the correctly encoded result only when edited. Do not
replace it with the u8 sRGB picker, which loses existing float precision, or
change serialized colors to linear. Canonical fabelgeist-color already provides
`srgb_to_linear`/`linear_to_srgb`; an equivalent existing Bevy conversion at the
framework boundary is also valid. Select one actual shared owner/conversion;
do not add independent transfer formulas. This is a source-backed mismatch,
not a claim of executed visual validation.

## Sequential lens ledger

### 1. Producer propagation

Contract: semantic leaves survive constructors/defaults, serde, library copies,
controls, trim arithmetic, retained cache/PBR records and native/image outputs.
Current disposition: MAT-R1/R2/R5 remain incomplete. Exact earliest raw producers
are material/engraving/ornament/trim schema fields and clothing_material::pbr;
controls and catalog defaults mutate/construct them directly. Carry typed leaves
through the field inventory above, including unchanged tiling density producers,
ArmorComponentMaterial and tactical runtime. Native PassParameters, generated
WGSL, image encoding, Bevy and egui are real primitive boundaries. Dependencies:
shared owners/serde/UI policy first, then all actual receivers. Acceptance: no
internal primitive relay/re-admission or parallel old/new API; all named call
families compile against the same owners; existing cases retain their values.

### 2. Existing type owners

Contract: reuse requires equal meaning, arity, unit, range and authority.
Metres/Ratio/PerMetre are transparent unit-only Math leaves; record admission
must still enforce finiteness/bounds. PositiveLength is already available to
Armor/Creator and exactly matches geometric positive band dimensions. Existing
ArmorComponentMaterial is the PBR owner to consume/refactor. Generic Color is
RGBA with named-field serde and no encoded-space invariant; TextileColor and
procedural SrgbColor are u8; Seed is u64 named-record serde. They cannot be
blindly substituted for float RGB3/native32 words. Missing general angle and
fractional texel-span/native32-seed roles need the lowest real shared owner,
not temporary local equivalents. Dependencies: dependency direction and both
locks, plus complete users if an owner moves. Acceptance: document exact owner
fit and direct adapters, no quantization, altered RNG, needless new wrapper or
new dependency to a high-level building crate.

### 3. Admission contracts

Contract: every construction/deserialize/default/GUI update has the invariant it
claims; immutable admitted values remain stable through native use. Current
Metal/Engraving/Ornament/Trim are publicly editable raw recipes with explicit
validation; native ValidatedMetal and Cut protect baking. This is not proof that
the public fields/serde already enforce bounded scalar contracts. Introduce
checked decoding where a new type promises checks, or keep honest unit-only
leaves with authoritative aggregate admission. Preserve Metal's actual default,
all zero cases, bounded fractional repeats and strict finite checks. Retain
image extent/word/signed-wrap/sample-address admission and original causes.
Dependencies: aggregate relief shape and existing invalid-value fixtures.
Acceptance: no derive bypass/private unchecked invalid default; API-only fixture
adaptation preserves existing scenarios/oracles, with no added case/helper.

### 4. Invariant operations

Contract: period, narrowing, UV rescale, projection, count arithmetic and mutable
replacement preserve their admitted guarantees or fail without partial publish.
MAT-C1 is confirmed in armor_metal density rescale and Code Review is completing
its exact source/repair family. This inventory also requires admission/failure
policy for public Trim::narrowed and typed finite output in period/cell helpers.
Do not infer finite results from finite operands or add an arbitrary epsilon.
Native tile limits bound texel/word products; scratch count cap bounds five-draw
offsets. Preserve 32-bit LCG wrapping and all native float operation order.
Dependencies: density/cause review and source geometry policy. Acceptance: compute
and validate replacements before mutation; propagate failure through actual
catalog/trim callers; retain valid zero and geometric band breadth.

### 5. Aggregate states

Contract: source and supported relief agree, decoded image shape/pixels/recipe
agree, all maps match a single tile, UVs match output mesh order, PBR fields travel
as one named appearance, library admission occurs before publishing a copy.
Current Cut/EngravingImage immutable borrows and MetalTextures admission are
sound local owners. Public Engraving can represent Ornament+Normal, rejected only
later; choose payload-owning source variants or cohesive controlled admission.
Trim output's correlated mesh/UVs and preview depth handle/scale need named
records. Preserve Split's skin/morph/coordinate permutation with one order.
Dependencies: serde and complete PBR receiving API. Acceptance: no independently
mutable duplicate source/relief facts, swapped tuple members or stale layout
packet; explicit native transfer packets remain legitimate.

### 6. Type ownership

Contract: one lowest shared owner per meaning, with direct final schema/API.
PBR already crosses Armor, Creator and tactical client; it cannot be completed
by a Creator-only wrapper. Physical band leaves can reuse Geometry; generic
units belong in Math, color-space conversion in canonical Color/framework, and
native32 generator identity below its users. Specific motif/repeat/scratch roles
stay with their material owner when not equal to existing textile/hair roles.
Dependencies: Root/peer ownership composition for shared exports/manifests; no
source-edit permission is inferred from this report. Acceptance: direct imports,
all users updated, no compatibility alias/forwarder or duplicate implementation.

### 7. Redundant authority

Contract: bounds/defaults, recipe/projection and cache dependencies have a single
current authority. Metal default is already used by polished catalog PBR; keep
that link instead of reintroducing a copy. Controls repeat several material
bounds and millimetre conversion policy; consume named typed authoritative
ranges. Other catalog material colors/roughness are authored choices, not
automatically redundant merely because some numbers repeat. Native motif
parameters are a derived packet, not a second mutable motif state. BakedMetal's
path-keyed snapshot and first initialization outcome are deliberate current
lifetimes, not proof of live file freshness. Dependencies: record/range owner
selection. Acceptance: one bounds/default authority, read-only derived native
projection, explicit snapshot/cache policy and no speculative watcher.

### 8. Behavior preservation

Contract: retain every supported value, shape, bound, arithmetic order, optional
case, source error, image color space and RNG word protocol except explicitly
identified repairs. Native shaders keep the same count-vs-launch distinction,
LCG stream partition, RGB/RGBA transfer, wrapped bilinear coordinates, slopes,
truncation/saturation and stencil-free workgroup constants. Preserve -pi/pi
endpoints, strength up to four, slant up to three, tiles 0.25..64, zero scratches,
zero physical depth, plain decoration, independent trim/engraving and geometric
positive bands. Default field JSON scalar/array semantics should stay explicit;
any deliberate final schema change must be recorded, with no dual decoding or
legacy adapter. Dependencies: frozen shader/fixture source comparison and full
cause/density repair. Acceptance: existing hosted fixtures/checks on final SHA;
no claim of numeric/visual execution from this discovery.

### 9. Enum shapes

Contract: semantic choices own their payload; primitive flags remain only at
actual native/checkbox/wire adapters. Fillet lines is a carried semantic role,
so replace application bool with a named two-state owner and define its exact
wire true/false mapping. Motif and Relief already own shape-specific values;
keep those payloads, with a named derived native motif packet. Source/relief
correlation needs a final cohesive representation; an image may genuinely be
Height or Normal, an Ornament may not. Decoration engraving/trim are independent
axes and must remain separately optional; cache.used and is_plain are ordinary
predicates/history, not mechanical enum targets. Acceptance: impossible semantic
combinations rejected by final owner without erasing legitimate inactive/edit
state; native u32 flags still map to the same shader codes.

### 10. Derivability

Contract: mechanical derives preserve both invariants and representation.
Existing serde/Clone/Copy/PartialEq/Error derives are appropriate for recipe and
closed value owners. Custom Metal/Ornament/Trim defaults carry authored nonzero
policy and must not become zero-derived Default. ReliefSource Display performs
semantic path/procedural presentation; MetalGpu Debug intentionally hides device
internals. GpuInitializationError keeps the reviewed derived source and manual
Display. New transparent unit owners can derive serde; bounded owners may derive
through checked conversion and must not transparently deserialize unchecked.
No additional local handwritten trait boilerplate was shown necessary.
Dependencies: actual pinned derives and chosen leaf admission. Acceptance: exact
same public/wire semantics; no dependency added solely for cosmetic derive use.

### 11. Type granularity

Contract: distinctions prevent actual role/unit mistakes without a universal
quantity framework or one wrapper per literal. Ratio is not restricted to one;
Metres need not always be positive; signed origin is not roughness; float texel
span is not integer count; scratch/ornament/curve counts are not HairStrandCount,
SortItemCount, ScanElementCount, InvocationCount or WorkgroupGrid. Geometric
TrimBand and authored Trim bounds differ. Prefer one shared PBR appearance
record, small mesh/UV and depth records, and one native motif packet. Keep raw
pixel arrays, fixed mathematical vectors, dimensions and exact serialization/
framework tokens at the actual boundary. Acceptance: explain the mix-up each
new distinction prevents; do not add arbitrary lower bounds, quantization or
pass-through adapter modules.

### 12. Magic strings

Contract: every string occurrence is accounted by its actual owner or real text
boundary. ReliefSource uses open PathBuf; LibraryName owns open user names;
Motif display names and UI labels are presentation, while discriminants/variants
control behavior. Shader binding names, source fragments, serde snake_case tags,
PNG/glTF tokens, filenames and existing fixture strings are actual native/wire/
authored text, not missing application enums. Decoration path text is a real
editable egui buffer admitted to Path at the filesystem call; CLI already owns
PathBuf. Preserve typed errors until terminal UI formatting: Article/Inventory/
CharacterRecipe String barriers are MAT-R5, not a legitimate native boundary.
Dependencies: Code Review's complete cause graph. Acceptance: no prose-driven
branch/new status-string protocol; no closed filename/name vocabulary; all real
semantic identity owners retained through application APIs.

### 13. Magic numbers

Contract: every literal/extraction has a checked unit/role or concrete native
operation. Material/relief/ornament/trim bounds and defaults have the units and
zero cases in the field inventory; controls must consume the actual policy owner.
Generated shader literals are ABI widths/bit masks, normalized coordinates,
workgroup dimensions, seed generator constants or owned visual policy. Preserve
which are float texels, metres, ratios, counts, byte stride and word indices.
Pixel packing's 8/24/255, four-byte words, binding padding zeros, sine/cosine
pairs, current sampler endpoint arithmetic and existing authored fixtures have
concrete native purposes. This is not a blanket exemption for all numbers in
shader/private code: each meaningful material/native parameter is traced above.
No new arbitrary tolerance/epsilon or generic raw scalar helper is authorized.
Acceptance: typed producers reach real numeric boundaries, native values/order
stay exact, and actual final hosted quality checks enforce unchanged policies.

## Other repository rules to compose

MAT-R3: own cohesive Result aliases for MetalError, EngravingError,
OrnamentError, TrimMaterialError, TrimError and map-data error operations. Keep
MetalTextureSizeError versus layout errors distinct, trait error signatures
explicit, and native mixed anyhow results honest. Do not relabel all admission
failures as device loss or widen errors only to shorten signatures.

MAT-R4: move trim reexports before module declarations, DecorationLibrary before
impls, and export/textures.rs test module last with structs before impls. In
other touched owning files, preserve docs/attributes and move complete items;
source comparison must show body/fixture equality. Root Armor/Creator library
exports have wider existing mixed-owner order debt; changes there require an
explicit composed source inventory, not a blind whole-file replacement.

Extract a real responsibility before material growth: scalar/appearance policy,
source admission, native packet projection and UI adapters are separable. Do not
expand a near-500-line geometry owner with unrelated recipe/serde code, make an
include/section banner stand in for a module, or raise no-growth baselines. No
new production unwrap/expect is justified by these type changes.

## Connected repair families and acceptance dependencies

1. Decide shared unit/color/native32/count/range owners and final serde/API
   contracts before editing their producers. Preserve precision and actual
   semantics; reuse ArmorComponentMaterial rather than a competing PBR record.
2. Migrate material, engraving, ornament and trim recipes/defaults/validation,
   then all controls, library/serde records, operation returns and retained
   preview/export/runtime fields. Project primitives only at native/GUI/wire
   arithmetic. Fix MS-COLOR at the real picker adapter.
3. Migrate shared PerMetre and physical trim values through unchanged tiling,
   lacing, rim/split and Creator density/trim receivers; coordinate MAT-C1 and
   the narrowed-operation policy before claiming finite outputs.
4. Replace correlated PBR/trim/depth/validation/motif tuples with named records
   using exact semantic leaves. Reuse/move canonical owners, remove superseded
   implementations directly, and inventory all affected field literal users.
5. Complete typed source-cause propagation through actual recipe admission,
   loading/saving/generation and UI receivers with the independent Code report.
   Keep image decode/IO/native causes and cached first completed outcomes.
6. Apply aliases/item order and responsibility extraction with exact unchanged
   shader and fixture body receipts. Existing fixtures may only adapt API;
   introduce no tests, cases, helpers, assertions or doctests.
7. Re-run all thirteen complete current-family waves, independent reviews and
   required hosted checks on the same final pushed revision. These discovery
   cells and source comparisons are not clearance or execution evidence.

## Explicit incomplete regions

The JSON source accounts distinguish full source from selected current functions
and metadata. Unread geometry/system portions of large clothing, parametric,
main, inventory, character-morph and UI files are not cleared. Root must finish
those portions relevant to its actual chosen API before changing them; Code and
Comment reports may supply independent coverage but must identify exact bytes.

If ArmorComponentMaterial's leaves/representation are changed, the identified
textile/closure/hook/lacing and tactical runtime field producers/consumers must
all be composed. Selected functions are read here; entire unrelated fastener
geometry, every textile algorithm and tactical runtime tests are not. The
component appearance owner's choice is therefore a concrete dependency, not
permission to silently change all downstream representation.

Actual material/engraving/ornament/trim native operations have been read, but
this report does not clear the entire GPU facade, host_float numerical library,
all generated-program roles, complete geometry admission, physical simulation,
all image formats, external asset freshness, or final CI. No unobserved Viewer/
Dataset material path is fabricated. The independent cause/density review and
Comment consumer inventory remain required inputs. Existing async/material and
native-first scoped approvals remain increments; they do not close these
unresolved semantic families.

## Fingerprints

Every exact current repository/dependency fingerprint and read extent is in
`/tmp/pr780-material-semantic-all13-discovery.json`. The external primary source
receipts pin egui 0.34.3 ui.rs and widgets/color_picker.rs, matching the actual
Creator Cargo.lock. The account includes both workspace and Creator locks and
Armor/Creator/Math/Geometry/Color manifests; no dependency mutation was made.


## Root design receipt before implementation

Root explicitly supplied the following design decisions after reading the
inventory; no implementation had started. They refine the proposed repair,
without supplying clearance for unread receivers.

- Reuse Math Metres/Ratio/PerMetre. Add one portable unit-only scalar Radians in
  Math. Reconcile the building generator's stricter finite/frame-owned Radians
  by composing the shared leaf while retaining its stricter admission and
  role. Do not add a second general angle implementation or import the
  high-level building crate into portable material code.
- Add a distinct native u32 seed word owner to Determinism. Preserve scalar JSON,
  all 32 bits, zero, wrapping native LCG arithmetic and existing draw order. The
  existing u64 Seed retains its independent contract; introduce no new RNG.
  Determinism already has serde and has no direct thiserror dependency; the
  unit-only word needs no new validation-error dependency.
- Add float encoded three-channel Srgb and four-channel Srgba owners at Color,
  distinct from unqualified RGBA Color and higher-level u8 palette colors.
  Preserve scalar array wire precision and existing color-space meaning.
  Resolve MS-COLOR only at the real UI adapter, with no quantization and no
  conversion on an unchanged editor response.
- Retain material terminal finite/range admission by immutable borrow; unit-only
  Math/Color leaves do not independently claim all application bounds. Introduce
  admitted bounded roles for scratch-per-tile count (zero permitted), motif
  repeats and Guilloche strands. One authoritative range owner must feed both
  admission and controls; do not create independently adjustable duplicate caps.
- Promote/reuse ArmorComponentMaterial as the one named PBR owner throughout
  catalog, components, retained records and callers. Do not add a second
  Creator Pbr record or tuple/primitive bridge. Final name/location and its
  construction/receiver frontier remain deliberately incomplete below.
- Close the Engraving source/relief state so Ornament+Normal cannot persist.
  Migrate controls, serde, Cut and native consumers directly to that final
  schema. Keep no compatibility fields, aliases, dual implementations or
  fallback decoder. Exact scalar/array/enum representation decisions must be
  recorded before source repair; the earlier current-wire inventory is not a
  demand to introduce a legacy adapter.

## Exact broader PBR and construction frontier

The following is a source-discovery frontier, not search-based clearance. It
identifies the actual owners, constructors and retained receivers Root must
compose before calling the chosen PBR API inventory complete.

| Frontier | Current inspected evidence | Remaining obligation |
| --- | --- | --- |
| Canonical PBR owner and producers | components.rs fully read; TextileColor::material fully read; TextileAppearance::material selected source read; hook/ring/fastener PBR constructors selected source read | Before changing owner leaves/name/location, inspect all associated construction and mutation routes in textile_pattern.rs, cloth_fastener_mesh.rs, cloth_hook_mesh.rs and cloth_lacing_mesh.rs. Preserve current u8-to-float channel division, opacity and metallic/roughness values. |
| Catalog appearance | clothing_material::pbr and armor_metal::metal fully read; GarmentSpecification fields and producer selected source read | Carry one appearance owner through retained GarmentSpecification instead of unpacking into new raw fields. Review any later clones/mutations in clothing.rs; no geometric fitting clearance is implied. |
| Creator preview and cache | preview.rs, metal_preview.rs and underlayer_preview.rs fully read; studio_generation material method and armory_scene fully read | Keep actual Bevy color/ratio projections at material construction, including component override, lacing, trim and no-texture fallback; preserve cache lifetimes. |
| Inferred textile receiver | textile_surface.rs:1-105 newly read, especially material() receiving fabric.material() | This consumer does not spell ArmorComponentMaterial but receives it by inference. Its base_color/metallic/roughness projection must migrate with the owner. Remaining texture/PNG helpers are unreviewed for unrelated behavior. |
| Portable/rigged output | export/rigged.rs and export/shell.rs fully read; full current export.rs/textures/equipment_export/character_export were directly read in the immediately preceding source review and same bytes are fingerprinted | Named appearance plus correlated surface/texture packet must propagate through character_morphs.rs, garment/export.rs, character_export/hair.rs and export compact/assembly/morph rebuilding, not just the initial Writer. Selected material-producing character_morphs functions are read; other named paths below remain unread. |
| Tactical runtime | equipment/runtime_equipment/meshes.rs fully read, including inferred component.material and equipment_pbr fallback | Adapt both material partitions and fallback atomically. Existing mesh_tests.rs must only receive API changes; its cases/oracles are not reviewed here. |
| Existing fixture receivers | material/engraving/ornament/trim fixture cases and preceding current metal/export fixtures were read | Existing textile_export.rs, export/textures/tests.rs and tactical mesh_tests.rs are identified but unread. No test, helper, case or assertion may be added. Preserve all existing valid/invalid scenarios when bounded count admission moves. |
| Construction and trim | construction.rs plus trim owners/edges/split/tests fully read; ParametricEquipment first 245 lines, actual selected trim path and armory trim source read | Finish parametric_equipment.rs tail (including its second trimmed caller/fixture) before changing the return API. Keep Construction::trim_on and Plate::widest_trim's current minimum behavior. |
| Shared density producer | tiling/mod.rs actual tiles chart function and tiling/lacing.rs length-to-chart projection read | Read whole surrounding production functions before a typed density change; preserve f32 multiplication/order, scatter and unchanged lacing native units. Existing tiling/tests.rs positive-band receiver is identified but unread. |
| Cause/serde | Decoration/library owners and selected Article/Inventory/CharacterRecipe validators read | Root must compose the Code agent's complete cause/density report and Comment agent's consumer inventory. Other source coverage is not inferred from those agents' names or historical reports. |

Identified but not fully read source dependencies now have explicit SHA-256
accounts marked unreviewed in the JSON: garment/export.rs,
character_export/hair.rs, export/{compact,assembly,morphs}.rs,
export/textures/tests.rs, tests/textile_export.rs, tactical runtime
mesh_tests.rs and Armor tiling/tests.rs. This list is a receiving API frontier,
not a declaration that no other inferred receiver exists. Source searches were
used to locate paths; direct receiving source and exact chosen API determine
closure. Root acknowledged that this shared-owner/API/PBR construction frontier
remains incomplete before implementation.
