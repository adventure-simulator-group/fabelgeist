# Remaining material physical family: current source discovery

Pinned HEAD: `ead46619dec5ef45993f63202cb289bd3986b018`.

This is read-only preimplementation discovery, not a review clearance. All
thirteen lens cells below were reconciled against the current core sources.
Whole connected-family discovery remains **incomplete** at the explicit
frontier. No source was edited, no project code/check/test/probe was executed,
no fixture was added or altered, and no CI/publication was requested. Prior PBR
reviews and historical ledger verdicts were used only as source maps.

The JSON ledger contains 122 current source accounts with SHA-256 and exact read
extents, plus 41 unrefreshed historical frontier accounts. Final fingerprint
reread matches every recorded current source account. Separate dirty generated
Map/shared-signature changes are outside this material report.

## Exact ownership

target/pr780-audit/handoff-material-async-scalp-first-f2c7422fd/manifest.json
protected_native and canonical_api; parent explicit latest agreement

Root owns whole MetalGpu::open, textures and read plus actual bake/record
functions. #826 retains new/batch/upload/scratch/dispatch, MetalDeviceError and
device_error. Typed recipe/count production can reach the existing native
signatures without changing those protected bodies. Coordinate any native facade
or shader-generator signature change explicitly before editing.

## Findings and connected repairs

### MP-1 — producer-migration-gap

Carry each physical or dimensionless quantity from the actual
recipe/default/serde/UI producer through retained state to its native
numerical/framework boundary.

- material.rs:22-38,75-119: scratch count/shape/angles/u32
  seed/waviness/grain/smudge remain raw; scratch_length is fraction of tile,
  scratch_width is floating texel span, waviness metres.
- construction.rs:29-59,195-280: Plate dimensions and Lacing.radius are metre
  scalars, hole_pairs is a physical pair population, ratios are independent
  unitless fractions.
- engraving.rs:25-33,66-86: depth metres; strength/repeats-per-tile/recess
  offset ratios; rotation radians. ornament.rs:13-44: repeated figures, strands,
  ratios and fillet role.
- trim/finish.rs:16-94 and trim.rs:32-46: authored width/physical
  band/period/nearest-rim coordinates are raw lengths; DepthMap.uv_scale in
  material/textures.rs:24 is a UV ratio.
- garment_material.rs:31-42,72-99: diameters/pitch metres, tilt degrees, repeat
  two-axis metres, density rings per square metre.

**Connected repair:** Choose the exact leaves, type every retained recipe field
and semantic return together, adapt defaults and existing literals, carry
through cut/image/sampling/trim/construction/mail chart, then convert only at
egui/image/Bevy/WGSL/numerical kernels. Also migrate Metal::TILES_PER_METRE and
dimensional bounds; never fix only PassParameters insertion sites.

**Dependencies:**

- Math units::Metres/Ratio/PerMetre already imported by Armor and Creator;
  Geometry positive/signed finite leaves have different acceptance policies.
- Plain radians/degrees, floating texel span, exact32 LCG seed and physical
  scratch/repeat/strand populations need an explicit canonical owner decision;
  MP-2.

**Acceptance:** No raw equivalent is retained between domain APIs; accepted
recipe bounds and valid zeros remain identical; field getters terminate at
documented arithmetic/serde/framework/native operations, not another application
wrapper.

### MP-2 — design-dependency

Unit, role, numeric policy and representation all have to fit before reusing an
owner.

- math/units.rs:14-38,52-100 provides unbounded Metres, Ratio and PerMetre; no
  bare Radians or Degrees.
- geometry/length.rs:16-39 rejects zero/nonfinite/negative; measurement.rs:21-65
  SignedLength rejects nonfinite. These cannot replace zero-valid
  depth/waviness/gap or unvalidated authored holders indiscriminately.
- determinism/seed.rs:54-94 is u64 record serde and eight-byte input; stream.rs
  uses SplitMix64, whereas gpu/metal/wgsl.rs:39-84 defines wrapping u32 LCG and
  top24 draw.
- building-generator/spatial_geometry/orientation.rs Radians is a finite
  generator geometry owner with higher-layer errors, not an importable low-level
  Armor leaf. Armor design.rs Milliradians is nonnegative quantized u16.
- mesh/count.rs describes native topology; gpu buffer/stride.rs describes native
  storage; pass/shape.rs and grid.rs describe launches. None describes authored
  scratch population or RNG draw offset.
- textile_pattern.rs:52-58 TextileRepeatCount is u8 textile-chart repetition
  with per-use bounds; TextileThreadCount is yarn population. Neither owns metal
  engraving geometry.

**Connected repair:** Reuse unit-only Metres/Ratio/PerMetre where exact; name a
coherent common angular leaf at the lowest shared layer if moving/reusing the
finite angle owner, or document a distinct unbounded-unit contract. Keep exact32
finish seed/LCG role distinct from world/root Seed without a widening/narrowing
adapter or alternative RNG. Use purpose populations, not storage/launch
wrappers, for authored scratch/repeat/strand counts.

**Dependencies:**

- A finite-angle owner move would affect existing generator callers and Geometry
  error/reflection policy; do not copy the same checked owner into Armor.
- A new shared deterministic32 leaf changes dependency edges if placed in
  determinism; both workspace lockfiles then need the exact Armor dependency
  edge. Local metal-specific native seed is a distinct role, not a duplicate
  64-bit root.
- No generic inverse-area/rings-per-square-metre owner was found in the
  inspected lowest owners; do not reuse PerMetre.

**Acceptance:** All accepted u32 seeds preserve exact bits and current JSON
scalar; all existing native coefficients and LCG draws are unchanged; invalid
angular/physical policy is not strengthened accidentally.

### MP-3 — aggregate-and-closed-role-gap

The source variant owns the relief payload combinations that can actually
execute.

- engraving.rs:25-28 independently stores ReliefSource and Relief;
  validate:110-132 rejects Ornament+Normal; Cut::new validates again before any
  native execution.
- metal_controls.rs:122-175 changes source and relief together only when
  selecting Pattern; Image can select Height/Normal. Inactive data transitions
  have current explicit policy.
- ornament.rs:20 fillets is a semantic carried choice; words:91-99 encodes the
  motif as positional (code,first,second,strands), and gpu/metal/ornament.rs
  duplicates code0..5 in WGSL.
- tiling/lacing.rs:31-36 already owns Lamellar/Scale Style. Cord.closed and
  through(leaving:bool) carry closure and travel direction; ordinary comparison
  predicates and shader u32 flags are boundary facts.

**Connected repair:** Select a final payload enum such as image(path+image
relief) versus ornament(motif+height depth), retain common sampling fields once,
propagate constructors/serde/UI/Cut/errors/native projection. Replace semantic
fillet and carried cord-direction/closure booleans with named roles at their
proper shared owner; keep true predicates and native flags at adapters. Replace
Motif::words positional tuple with one named native projection derived from the
actual motif, and give Rust/native codes one authority.

**Dependencies:**

- Do not reuse TextilePathClosure blindly for plate cords: its owner includes
  textile stroke semantics. A shared curve-closure move has its own callers.
- Engraving existing bounds fixture deliberately constructs Ornament+Normal.
  Plan direct existing-case API adaptation before changing representation; no
  new test, case, helper or assertion and no deleting its semantic obligation.
- Any GPU generator facade/signature change needs exact #826 agreement;
  protected MetalGpu methods can remain unchanged.

**Acceptance:** Unsupported Ornament+Normal remains impossible/rejected rather
than silently enabled; every currently valid source has identical decoding and
native output; motif code0..5/payload/order, neutral lanes and fillet behavior
are preserved.

### MP-4 — causal-validation-gap

A material failure keeps its authoritative typed cause until final presentation,
including recipe/load/save/application paths.

- inventory.rs:174-198 stringifies ItemDesign, Decoration, Construction,
  GarmentSelection and SettledGarment errors; Inventory::validate:353 returns
  String; lib.rs:204-224 CharacterRecipe::validate returns String.
- generation.rs:7 and main.rs:230,364,378 call Error::msg on those strings. The
  underlying EngravingError/MetalError/TrimMaterialError/ConstructionError can
  no longer be downcast or traversed as sources there.
- garment_material.rs:111-166 owns finite/bounds/dependent interlinking
  admissions via anyhow::ensure messages, with no stable typed MailWeave error
  classes.
- library_name.rs:12-20 TryFrom returns String; decoration_controls.rs:16 and
  wardrobe_tab.rs:24 wrap it using Error::msg.
- garment/selection.rs:207-225 wraps coif failure into formatted Debug prose,
  unlike the typed Context path for sewing pattern; MP broader cause frontier
  remains incomplete.
- outfit.rs:17-21 turns typed loadout conflict into presentation text inside
  anyhow. Final studio_ui output correctly formats at UI, but the cause has
  already been erased.

**Connected repair:** Introduce or reuse typed authoritative validation errors
at MailWeave/LibraryName/Recipe/Inventory/Article as needed, retaining
item/name/version/count context and sources. Keep actual lower
design/engraving/construction sources via from/source/context; remove internal
format-to-String and Error::msg barriers through generation/main/load/save and
library receivers. Keep presentation at final UI/CLI/warning boundaries.

**Dependencies:**

- Preserve exact first-error order: recipe version, coefficient counts, name,
  nonfinite coefficients, inventory order/duplicate IDs then article validation
  order; Catalog design then decoration then construction, Garment resolution
  then construction then drape then mail.
- Actual ItemDesign and ParametricDesign validators mostly already preserve
  typed errors via Error::new/Context; do not replace them with broad new
  message adapters.
- Fresh broader pattern/stage/equipment_catalog/occupancy/body-proportion
  validation source remains explicitly unread, so this report does not certify
  whole application causal closure.

**Acceptance:** Actual source chains expose original typed
material/IO/image/native/geometry causes after recipe and library handling;
unchanged finite/range checks and first failure survive. Only UI/CLI text
changes presentation, not control flow or unsupported acceptance.

### MP-5 — fallible-publication-gap

Draped mail preview must return source failures and publish cache/applied state
only after successful map generation and complete decoding.

- drape_preview.rs:80 expects MailMaps::new and :96 expects PNG decode;
  material/material_for return StandardMaterial infallibly.
- callers are poll (selection lookup around186), refresh_mail:266-304, and
  wardrobe_tab.rs:292-312 spawn. This is the separate mutable draped-mail
  family, not the corrected underlayer MailImages.
- underlayer_preview.rs:107-137 already decodes color/normal/occlusion fallibly
  before adding assets; its mail cache is inserted only on success.
- drape_preview.rs:118-124 evicts/pushes recent material only after
  construction, but the current eager image-upload closure can add an earlier
  image before a later decode fails; refresh_mail records applied weaves after
  loop.

**Connected repair:** Make map/material generation fallible, decode every
selected map before Assets insertion/cache mutation, carry Result through
material_for, poll, wardrobe spawn and refresh handling. Preserve old materials
and retry eligibility on failure; do not advance applied weaves after failed
replacement. Keep recent-cache capacity4 and success eviction order unchanged.

**Dependencies:**

- Preserve white scalar factor, already encoded PNG color vs linear
  normal/occlusion, Repeat sampler, Mask .5, double sides and per-axis ring
  repeat UV transform.
- Do not change process OnceCell success/failure caching or native retry
  semantics; that policy is already explicitly owned separately.

**Acceptance:** All three actual caller paths handle causes at final UI; no
production expect, no partial asset/cache/applied publication for failed
replacement, identical successful materials and PNG bytes.

### MP-6 — behavior-and-redundant-authority-gap

Editing ring appearance must not invalidate cloth simulation, while true
layer/order/body/lining/drape inputs still do.

- garment_material.rs module docs and garment/selection.rs:122-126 explicitly
  call mail appearance-only; same_simulation:199-205 compares
  construction,fabric,resolution,drape and excludes mail.
- inventory_article_editor.rs draped return uses same_simulation/layer, so mail
  edit intentionally does not set dirty; drape_preview::refresh_mail updates
  only materials.
- studio_cache.rs:148-162 drape_changed serializes full piece.selection,
  therefore includes mail; studio_generation.rs:117-121 invokes it on next
  regeneration and requests a drape if changed.
- outfit.rs:53-79 computes under-plate standoff/order from loadout, fabric,
  channel and lining; these genuine dependencies cannot be dropped just because
  same_simulation omits layer.

**Connected repair:** Let GarmentSelection own an explicit simulation-input
projection shared with same_simulation/cache identity; retain actual independent
routing layer/order/settled drape/body/lining dependencies once. Exclude
MailWeave appearance from that simulation key and retain live name/appearance
updates without recomputing dynamics.

**Dependencies:**

- The actual cache/worker checkpoint source must be read before selecting final
  key; garment/stages.rs and drape worker internals are not freshly complete
  here.
- Do not use a fresh hand-copied list in StudioCache that can drift from
  same_simulation.

**Acceptance:** A mail edit followed by an unrelated regeneration does not
re-run cloth stages; construction/fabric/resolution/drape and actual
layer/order/lining changes still request the appropriate existing simulation.

### MP-7 — connected-geometry-admission-defect

Construction and trim operate on agreeing finite, index-valid columns; physical
recipe admission alone proves no mesh shape.

- design.rs:134-160 exposes all GeneratedArmor columns publicly;
  surface_grid.rs:9-17 rows/columns/vertices are independently public.
- tiling/mod.rs:53-77 validates Construction then GridSurface::is_spanning;
  surface.rs:115-131 only checks rows/columns/product and directly indexes
  positions by grid IDs.
- Representable counterexample: a valid Scale(Tiling::scale()), a public
  regular(2,2,false,vec![0;4]) grid, and empty position/normal columns passes
  is_spanning then positions[0] panics.
- trim.rs:88-99 checks nonempty faces and triple count, then Rims::new. Empty
  positions plus indices [0,0,0] and [PlateFace::Outer] passes that count
  condition then trim/edges.rs plates/rim_edges indexes welded[0].
- trim/split.rs:42-75,210-252 indexes normals/UV/skin/morph/component ranges
  without a complete source-column admission; tiling/embedded.rs:62-104
  similarly consumes skin and morph columns.

**Connected repair:** Add operation-local borrowed admitted geometry views or
consume an existing canonical checked source, checking only used columns,
complete triples, finite native coordinates and all referenced
IDs/component/morph extents before indexing or mutation. Propagate actual typed
causes through Constructed/Trim error owners and callers; no global geometry
algorithm rewrite.

**Dependencies:**

- Separate connected family from already inventoried GLB/export admission
  UV-R1/R2; validating an exporter does not repair these earlier public CPU
  consumers.
- Native GeneratedArmor producers and shared SurfaceGrid construction/recording
  functions remain a dependency requiring exact ownership/caller inventory
  before repair. This report has not freshly read all producer paths.

**Acceptance:** Malformed representable input returns an owned typed error
before indexing/partial mutation; valid construction/trim topology, UVs, morph
order and native bytes remain identical; no new recipe ceiling.

### MP-8 — physical-count-operation-defect

The advertised plate budget and native index limits must be checked before
row/column growth or narrowing.

- tiling/mod.rs:29 MAX_TILES=20,000; layout.rs:83-87 checks layout.tiles.len
  only after lay_grid returns.
- layout.rs:113-129 converts ceil row population to usize then adds1 and
  allocates grid_rows; columns:220-250 collects physical columns before checking
  remaining budget.
- Representable finite well-shaped planar 2x2 surface with height1e18m and
  width1m has finite frame/cross arithmetic but row quotient exceeds 64-bit
  usize for default Scale pitch; cast saturates then +1 overflows before
  TooManyPlates. The 32-bit path fails sooner.
- tiling/mod.rs:102 and tiling/lacing.rs:253,298 cast grown vertex populations
  to u32; template and trim/split also narrow without a shared admitted full
  mesh extent.

**Connected repair:** Admit physical row/column/tile/cord populations with
checked host arithmetic and remaining actual MAX_TILES budget before
allocation/growth; use canonical
MeshVertexCount/MeshTriangleCount/MeshCornerCount for native topology boundaries
and reserve causes where actual allocation is fallible. Never substitute
InvocationCount/BufferElementCount/sorting lengths for those physical
quantities.

**Dependencies:**

- Exact existing small-plate budget is already policy; this is enforcing it, not
  authorizing a stricter geometry extent cap.
- Full finite-geometry and native producer frontier must be inventoried before
  implementing count/shape changes; no speculative altered tiling algorithm.

**Acceptance:** Oversized construction returns TooManyPlates or an exact
representability/reservation cause before growth; accepted small geometry and
every existing face/index order stay unchanged.

### MP-9 — mechanical-derive-and-prose-classification

Keep only trait implementations whose semantics differ from an available derive,
and classify raw prose at owning boundaries.

- library_name.rs:23-39 From/AsRef/Display are mechanical; Creator Cargo already
  enables derive_more display/as_ref/from. clothing/error.rs:6-10 demonstrates
  exact String-field #[as_ref(str)] and formatting-preserving Display
  forwarding.
- ReliefSource Display is semantic path-versus-ornament classification; MetalGpu
  Debug intentionally hides kernels, GpuInitializationError Display forwards
  Formatter flags to shared actual cause. These manual implementations are
  justified.
-
  MetalTextureSizeError/LayoutError/MetalError/EngravingError/OrnamentError/TrimMaterialError
  already use thiserror; do not replace typed errors with strings.
- Metal UI and motif shape sliders duplicate policy ranges already owned by
  validators; current exact ranges match, but shared typed bounds should become
  the authority rather than retaining competing literals.
- Retained library paths in Decorations/WardrobeLibrary are String, while actual
  IO expects Path; source path inputs in Args are PathBuf and Engraving already
  uses PathBuf. UI TextEdit raw text is a true boundary.

**Connected repair:** Use available equivalent derives for LibraryName
mechanical AsRef/Display if touched, retain checked TryFrom and exact trim
policy. Expose named typed bounds from semantic owners to UI. Carry canonical
filesystem path owners through internal state; raw edit buffers and display
strings stay at egui/IO formatting.

**Dependencies:**

- Do not change default material recipes via generic Default derivation:
  Metal/Lacing/Trim/Mail/Ornament defaults are authored construction data.
- Path/name/cause closure must include actual library save/delete and final UI
  consumers; open names/paths are not closed enums.

**Acceptance:** Wire strings and trim-whitespace behavior are identical, no
string classification drives material roles, all derive bounds/Formatter
behavior remain equivalent; policy ranges have one semantic owner.

## Native arithmetic and preservation contracts

- **metal_side:** 32..1024; non-power-of-two sides currently accepted;
  PREVIEW1024
- **texels:** side²<=1,048,576 logical records; exactly one u32 RGBA word per
  texel for native maps
- **slopes:** 2 words/texel<=2,097,152; finish3 words/texel<=3,145,728;
  MetalTextureSize.words checked native multiplication
- **scratches:** 0..3000; zero skips scratch dispatch; 5 draws/scratch, <=15,000
  draws
- **scratch_stamp:** ceil(length*side*2)+2<=822 most_steps; +1<=823;
  div_ceil64<=13 x workgroups; y<=3000
- **bake_noise_skip:** scratch_draws+index+1<=15,000+1,048,575+1=1,063,576,
  independent of wrapping LCG state arithmetic
- **finish_lattice_skip:** layer0..4; stride1,048,576, maxused layer4
  with7x7:4,194,304+6*7+6+1=4,194,353; layer2 3x400 maxskip2,098,352, all within
  u32
- **image_extent:** each side>0 and 2*side-1<=i32MAX (side<=1,073,741,824);
  total words w*h*(1 or2)<=u32MAX; floor sample and floor+1 separately checked
  before upload
- **mail_image:** width256; admitted row_pitch<=outer/2 implies tile_y<=outer,
  tile_x=outer+.0001>outer, hence rounded height<=256 and .max1; <=65,536
  texels/262,144 RGBA bytes per map. No need increase ceiling or add GPU
  dispatch.
- **mail_chart:** wire>0 and outer>=4wire ensure positive centre/wire radii;
  tilt0..60 means finite sin/cos; raw raymarch128 steps, tolerancewire*.001,
  probes5 reach3wire retained.
- **trim:** authored width .003..=.06m; operation TrimBand separately admits any
  finite positive width/period, not only authored bounds; do not accidentally
  impose authored bounds on valid generic trimmed input
- **construction:** platewidth.012..=.15m,height.02..=.2m,
  thickness.0005..=.012m; gap/hole radius/bevel can be0; overlap0..=.6,
  stagger/roundness0..=1; pairs0..3; lacingradius.0004..=.003 and <=.9hole
  radius; tiles<=20,000 policy checked too late today

## Thirteen sequential lens cells

1. **producer-propagation** — MP-1, MP-4, MP-5.
      Recipe/default/serde -> retained Metal/Lacing/Weave/Engraving/Trim ->
   editor/mutation -> CPU/native bake -> maps -> preview/GLB read through
   current owners.
      Acceptance: No .metres/.ratio/seed extraction then re-admission across
   application fields; actual final projection boundaries named.

2. **existing-type-owners** — MP-1, MP-2.
      Compared Math unit-only leaves, Geometry finite/positive measurements,
   64-bit determinism seeds, exact GPU storage/launch owners, Mesh topology
   counts, higher building angle, image byte packet and textile-specific
   quantities.
      Acceptance: Owner unit/frame/policy/wire agreement documented; missing
   owner is explicit, not disguised by a wrong wrapper.

3. **admission-contracts** — MP-2, MP-3, MP-4, MP-7, MP-8.
      Public fields/literals/serde are authored holders; ValidatedMetal
   immutable borrow admits full recipe; Cut/Image constructors and native
   image/address admissions preserve exact finite/bounds policy; public mesh
   factories still bypass used-column shape.
      Acceptance: All routes/defaults/serde/UI and operation-local views keep
   accepted domains and exact sources; untraced lower producers remain
   incomplete.

4. **invariant-operations** — MP-1, MP-5, MP-6, MP-7, MP-8.
      Inspected scratchdraw/step arithmetic, shader wraps, signed image wrap,
   trim period/narrowing, dependent holes/mail, row/column allocations,
   map/cache publication and actual retained state.
      Acceptance: No overflow/failed operation publishes bad state; preserved
   native algorithms and legitimate zeros/clamps; exact high-count failures are
   owned.

5. **aggregate-states** — MP-3, MP-5, MP-6, MP-7.
      Source+relief correlation, immutable Cut with actual image layout,
   lacing/holes, wire/opening/pitch, map extents, UV authority, simulation keys
   and per-morph source columns.
      Acceptance: Atomic source admission/enum states retain relationships; leaf
   wrappers alone are not certified geometry.

6. **type-ownership** — MP-1, MP-2, MP-3.
      One existing Math/Color/Geometry/Mesh/GPU owner per actual meaning; no
   high-level Angle import or 64-to32 seed truncation; Root/#826 exact function
   graph recorded.
      Acceptance: Dependency agreement before protected shared mutation; no
   duplicate formulas, temporary adapters, competing layouts/RNG.

7. **redundant-authority** — MP-3, MP-6, MP-9.
      Motif code table vs native constants, duplicate range literals, actual
   Metal/Mail cache equality+inputs, source identity and simulation key vs
   same_simulation.
      Acceptance: One role/policy/source projection; caches own observations,
   not a second independent current fact; failed state does not advance applied
   keys.

8. **behavior-preservation** — MP-1, MP-2, MP-3, MP-5, MP-6, MP-8.
      No repair yet: current native program/coefficients/rounding/wrapping/order
   and existing fixture oracles recorded as baseline, including all accepted
   zero and bounds policies.
      Acceptance: Exact source reversal/native WGSL comparison after API
   migration; existing tests only; hosted validation later by Root; no inferred
   visual/runtime acceptance.

9. **enum-shapes** — MP-3, MP-4.
      ReliefSource+Relief forbidden pair; fillet choice, cord direction/closure;
   existing Style, Construction, map roles, Source/Pixel enums; predicates and
   required shader u32 flags distinguished.
      Acceptance: Closed semantics carry only their valid payload, independent
   facts remain independent; arbitrary ordinals/paths are not enums.

10. **derivability** — MP-9.
      Inspected all manual trait/default implementations in reviewed core,
   LibraryName actual derive dependencies, current error derives and meaningful
   GPU/source/error formatters.
      Acceptance: Mechanical derive retains exact bounds/format/serde policy; no
   new unchecked Deserialize or authored Default substitution.

11. **type-granularity** — MP-1, MP-2, MP-3, MP-8.
      Distinguished metres/UV ratios/floating texel spans, scratch counts/RNG
   draw offsets/mesh counts/storage elements/launches, authored holders/admitted
   views, native numerical frame values.
      Acceptance: No universal quantity framework, per-numeric-value enums, or
   role erasure; numerical arrays remain at demonstrated kernels/framework APIs.

12. **magic-strings** — MP-4, MP-9.
      Core source strings classified into owned error prose, path/library
   identity, UI labels/IDs, native PassParameters/WGSL field and entry names,
   serde syntax, GPU labels, authored/fixture values; offline Python Trim is a
   separate native GLB postprocessor, not Rust Trim serde.
      Acceptance: No branch on mutable human error prose; authoritative causes
   retained; PathBuf/LibraryName at internal identity; native shader/PNG/glTF
   schema strings remain at actual adapters.

13. **magic-numbers** — MP-1, MP-2, MP-3, MP-8.
      Every core declaration/default/operation/native constant reviewed by
   field/unit/population/protocol category; policy and physical constants
   tabulated; ordinary indexes/dimensions/padding and fixed numerical kernel
   coefficients distinguished.
      Acceptance: Typed quantities originate at source; preserve numeric
   constants/op order and accepted bounds; counts have correct authority; all
   external frontier source occurrences are explicit unread.

## Intentional and boundary dispositions

- ValidatedMetal and immutable image/Cut borrows already retain source
  admission; no new branding needed solely because recipes are public authored
  holders.
- MetalTextureSize and MetalTextures already own logical extents/native packets;
  GPU generic RGBA8 Image is not an appropriate replacement for f32 ReliefImage
  height/slope plus signed sampler proof.
- Finite Color/Ratio policy belongs to recipes, not unit-only leaf constructors;
  Srgb/Srgba current owners are retained from committed scalar migration, not
  duplicated.
- Decoration engraving and trim Options are independent valid facts; Constructed
  plates+lacing are separate generated outputs; do not collapse those axes.
- Existing OnceCell<Result<_,GpuInitializationError>> first-outcome process
  policy is intentional; Arc cause and manual Formatter delegation are
  preserved.
- Native RGBA layouts, host_float ZERO hook, field strings, GPU labels, actual
  draw/dispatch/pad literals, shader numerical coefficients, authored default
  proportions, array dimensions and existing numeric fixtures are concrete
  native/authoring boundaries; their semantic policy bounds still originate
  once.
- Python trim_armor.py consumes native GLB plate-edge metadata in metres, has
  separate typed-by-validation JSON style protocol (1..30mm, bands100mm and
  integer repeats1..128); never feed Rust Trim/Engraving serde into it or
  silently merge its distinct paint/slab policy.
- No new test/case/helper/assertion/doctest is authorized; fixture source reads
  establish existing obligations only, not execution or new acceptance.

## First unfinished cells

- Whole-family producer/admission/aggregate/cause graph: remaining actual
  validation functions in garment/pattern.rs and stages.rs,
  inventory/occupancy.rs, equipment_catalog.rs, character-proportion owner and
  their contextual readers must be freshly read or explicitly scoped out with
  source before all13-complete.
- Shared geometry source/dependency graph: SurfaceGrid native producers and Used
  GeneratedArmor columns/component/morph construction in gpu/body.rs,
  device_fit/fitted_existing/underlayer producers need fresh operation-local
  admission plan and exact shared-function ownership before MP-7/8 repair.
- Native/application export receiving is partially refreshed (RiggedShell, maps,
  writer and concrete call sites), but full
  export/compact/assembly/morph/validation and runtime consumers remain
  historical map only. Committed PBR source report is preserved; it is not fresh
  physical-family clearance.
- All source-wide raw string/primitive literal classifications outside explicit
  read extents remain incomplete. This report cannot be cited as exhaustive
  whole-family preinventory permitting arbitrary repair outside the populated
  core cells.

## Current source fingerprints

| Source | SHA-256 | Read extent |
| --- | --- | --- |
| crates/AGENTS.md | `2d9c7c48f7c9d02c2996ee71cd64c8dc97ef71ee260f99b16ff388a782f63f23` | [[1, 192]] |
| crates/fabelgeist-armor/AGENTS.md | `52390164bf3eb74a0160ee3ac19527fd43856aa998920d16824f2f41058cf47f` | [[1, 247]] |
| crates/adventuresim-character-creator/AGENTS.md | `b222ba01455443717c807738e311b12d8ded9a33cd95084b35870943357898a7` | [[1, 11]] |
| crates/fabelgeist-armor/src/material.rs | `ff72cece4156ed5f18aa3af3011c590eccae7e45340c755ad32e7b73339cf69e` | [[1, 134]] |
| crates/fabelgeist-armor/src/material/tile.rs | `a094fe52ceba98b71a00ae2fa974ba24cf769131e3c265373727f353f3885869` | [[1, 67]] |
| crates/fabelgeist-armor/src/material/textures.rs | `89a5b76b6d0a2783ce6dbacb7329e7a2cda93ddceaf4fc12c060e7ff2c96036e` | [[1, 100]] |
| crates/fabelgeist-armor/src/material/density.rs | `94db413bd0eeefd9fc860abb8eee368ee1d11a55cb48cbb4cd7ba2113fd0d665` | [[1, 122]] |
| crates/fabelgeist-armor/src/construction.rs | `c9b01e2ab8ba1fb7b0201e0d95ecb41ca101cf0f6bb56adab0ed656f61a59ff6` | [[1, 388]] |
| crates/fabelgeist-armor/src/engraving.rs | `c612e940bd0898395fad63b7d819dce7b863022f3258ade844f72d8a6a1c3dd2` | [[1, 288]] |
| crates/fabelgeist-armor/src/engraving/error.rs | `0a96086e52a505d0b70fb45480e744c08a6415a804a39dc8880d44748173e83e` | [[1, 45]] |
| crates/fabelgeist-armor/src/engraving/decoded_image.rs | `7ef1b572bf5a5c2e5684487da81194d1793c4025654f099c0898d97dfba064f1` | [[1, 75]] |
| crates/fabelgeist-armor/src/engraving/sampling.rs | `d042b91ea2f8b98dd53b7733dd4f5f11a282b4763211f3ca3288b2e18879d548` | [[1, 108]] |
| crates/fabelgeist-armor/src/ornament.rs | `74d886336dc4fbeeb4f9ff4ec29530a32fef42865dcb7ef2f6b2220b678c1ebf` | [[1, 194]] |
| crates/fabelgeist-armor/src/trim.rs | `d32e281322f275748d496491151217f53fab6e74b14260ca8766b98bc9aa13a6` | [[1, 130]] |
| crates/fabelgeist-armor/src/trim/finish.rs | `042db495d8b465dffdf12366970d62c57f929b796e2e9f8974f5f5bddd3921e4` | [[1, 160]] |
| crates/fabelgeist-armor/src/trim/edges.rs | `ffdc73a54de37023064d476bba8fbf666fe57f7597a910468c96fa3e9f63b2c7` | [[1, 319]] |
| crates/fabelgeist-armor/src/trim/split.rs | `44aafa5468b01108bee89840514846df79aa2075ea6c2e154b21d93cee321b9a` | [[1, 330]] |
| crates/fabelgeist-armor/src/tiling/mod.rs | `7ff5bfe0408afa8433957ee7575dd1fbb9f83d6afe4bb3af59a581a81724280d` | [[1, 137]] |
| crates/fabelgeist-armor/src/tiling/layout.rs | `9d9424e679eb33b176ce1687f202f9a81fcf765ef6d910fdd02bb1f409711fc2` | [[1, 342]] |
| crates/fabelgeist-armor/src/tiling/lacing.rs | `cce09840d83fc6c71d1020fc45961df2a0c9b4c2a85d5702bc0f65d6fad902a4` | [[1, 321]] |
| crates/fabelgeist-armor/src/tiling/surface.rs | `b11e892b1c5cebf91866d5ffeed3801659fd98201fe446753a33aadf65598020` | [[1, 441]] |
| crates/fabelgeist-armor/src/tiling/tile_map.rs | `2acac8447a7f7a9347382ff425f74de58a7aeef6786c43d1d352412cbaff4065` | [[1, 115]] |
| crates/fabelgeist-armor/src/tiling/template.rs | `a1cd00e8c02d5c25d675041927317755b24262122ac4096db8fec9a5b1ac16c8` | [[1, 317]] |
| crates/fabelgeist-armor/src/tiling/embedded.rs | `6c7f84f8e6b242d32e19ddecedfa1cfdb5f849bd984008eda7b0ea1da79e5345` | [[1, 115]] |
| crates/fabelgeist-armor/src/tiling/tests.rs | `166ec2be7855c465b709326152b2be1babfdcd538a062213cc6a5d8ae680d1f9` | [[1, 192]] |
| crates/fabelgeist-armor/src/pattern.rs | `8d0d262278bc2b6511da93bf24abf111ae4e5649b288166b56a2a5a4a17d1ca2` | [[1, 86]] |
| crates/fabelgeist-armor/src/surface_grid.rs | `d2e76894c4233cad8a8656215bf15fe6defcce235adc2350b3e2d5ad0308465e` | [[1, 108]] |
| crates/fabelgeist-armor/src/design.rs | `86dac554a42b48cf1547e5656337fb718b4a4fc7e231a30009ce00ff7207dd00` | [[1, 183]] |
| crates/fabelgeist-armor/src/gpu/metal/mod.rs | `b94c9bc6659a4ca3a7e9ef3bfbe5a52026c70dd86abd4c40f91c5066c9ebde8e` | [[1, 149]] |
| crates/fabelgeist-armor/src/gpu/metal/bake.rs | `6ef02db0cef5c45d6560976ce4f3ae011f591d755a9e0313ec3cc75c04f69a3d` | [[1, 264]] |
| crates/fabelgeist-armor/src/gpu/metal/textures.rs | `e6e7f6eb9a2ef8dbb4d28132efa4d82f95ef220730cc20d12ccd376c43a14a9d` | [[1, 323]] |
| crates/fabelgeist-armor/src/gpu/metal/finish.rs | `d49752ba70390e3801be29f335f22fea33532331b7e73c555474f8a55e866741` | [[1, 135]] |
| crates/fabelgeist-armor/src/gpu/metal/ornament.rs | `6a9f887c5c746b8b70fc8a3c9741ffa1a68dd3580a61483ca88fbc244dcb4152` | [[1, 198]] |
| crates/fabelgeist-armor/src/gpu/metal/wgsl.rs | `90083b6c258f47bcf6e52babf50abc2cda917c32ea982b0404edcd75b8157f83` | [[1, 69]] |
| crates/fabelgeist-armor/src/gpu/metal/tests.rs | `5784fb2e5a6b5c72be402465e6d28a84422e5274759c4674f560abe2aa8cebc4` | [[1, 259]] |
| crates/fabelgeist-armor/Cargo.toml | `a82f0e3c9912c2ee10b704021d81d667163eb83c6da16804ed8fcc6e23ece052` | [[1, 29]] |
| crates/fabelgeist-math/src/lib.rs | `bd588528cad05e62401905c31645e928a15b1175985219d9d79d21b2b2e81510` | [[1, 17]] |
| crates/fabelgeist-math/src/units.rs | `0c76a1a79a18e833d21fc2ffb011e2a351139ee7a299df13df12fdbcf3c27492` | [[1, 120]] |
| crates/fabelgeist-math/Cargo.toml | `6aaf0cb94d1603d0d6cdd07f1bee8021c3c29c841d01d144143faa4be39a6321` | [[1, 17]] |
| crates/fabelgeist-geometry/src/lib.rs | `4be1dea1bfca250604d52c9c8ea026aac5e3608d4d8cf35168f0cdc60ff4a7ab` | [[1, 16]] |
| crates/fabelgeist-geometry/src/length.rs | `7ce3b7a0720e800087df009c9726c0869988dfad3c24cc4a67bfbf49e059f48e` | [[1, 40]] |
| crates/fabelgeist-geometry/src/measurement.rs | `72ee58eb8ae01af4b7f17ef81f007dfa63c7dc818a1ed71ce5bbba0707236e70` | [[1, 104]] |
| crates/fabelgeist-geometry/src/error.rs | `78064ad3e76e6e39580db42af5da259e24a922d69fbe0953704cb9715f26a5c3` | [[1, 95]] |
| crates/fabelgeist-determinism/src/seed.rs | `ad0f8971f300e87ee98b0cd1d44273bb361c576566444a2b45216f9ddb65dff4` | [[1, 213]] |
| crates/fabelgeist-determinism/src/stream.rs | `c2ec90f6b96a13e2f97e9b20426954990d9df8a0331b08bb5709d73e0d09cec6` | [[1, 169]] |
| crates/fabelgeist-determinism/src/lib.rs | `85dcbab734e5f6b47c035628dac19ae5d331d078f7cbce23e3f5debd34986cdb` | [[1, 58]] |
| crates/fabelgeist-determinism/Cargo.toml | `c39a8a5055ca88572a587d0eadbde46eeb889d9060ec6b8930f59bfb76e44303` | [[1, 26]] |
| crates/fabelgeist-gpu/src/data/gpu/buffer/stride.rs | `9a8bfaa7b7b1d2d3c6c72d27cbd6f294c14356fee69bf72a59d4af38f89490b5` | [[1, 117]] |
| crates/fabelgeist-gpu/src/data/gpu/compute/pass/grid.rs | `4a34f1d75e228c0e8f5c9012eda4a8cdd7b99e79aebc3ef53cfaf061ebdf3ac6` | [[1, 109]] |
| crates/fabelgeist-gpu/src/data/gpu/compute/pass/shape.rs | `7998ee67967dc4ea5747e1396944628c1e9e89060c350d0e75d2a91bb85c81f2` | [[1, 141]] |
| crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/image.rs | `b7a403ce0f4a856e97457918a1b1fe9a0d4e34fc8365ce806107ff48a6d4d049` | [[1, 239]] |
| crates/fabelgeist-mesh/src/count.rs | `e609778b6bd24b4ee231f3d3281d463764a55d5b787f02f4015f4ff7c4b4072b` | [[1, 135]] |
| crates/fabelgeist-mesh/src/count/strip.rs | `92f8eede012a1018363a73fcd79a8f703862a3f6c40291a30e2cd02264e95ffe` | [[1, 136]] |
| crates/adventuresim-building-generator/src/spatial_geometry/orientation.rs | `05f48ed8a27959acd5930fc56690d454e5307482f673cbdb5628d88970c3db1d` | [[1, 142]] |
| crates/adventuresim-character-creator/src/lib.rs | `3014f331d32d092248326e49b6966aca09036b86b68220167f56924abf160d84` | [[1, 304]] |
| crates/adventuresim-character-creator/src/garment_material.rs | `098f2b193a044af8ad54a0517a96a2308c30ec0fd002c2e7f6905747eb2fd9c1` | [[1, 348]] |
| crates/adventuresim-character-creator/src/garment_material/rings.rs | `471ad91001a7b09374e81f3e5d774a7fa8f64ac51ca4b7aec860e0433e949d5d` | [[1, 258]] |
| crates/adventuresim-character-creator/src/metal_gpu.rs | `3b90085dce11463f05a859d535dae2006b41e3332441d7f407c8b2b8dd837535` | [[1, 22]] |
| crates/adventuresim-character-creator/src/gpu_initialization.rs | `10bc50519c97e93024e55b870b20dac09dc6bee84958a36366e2b9eb07cc5e01` | [[1, 27]] |
| crates/adventuresim-character-creator/src/metal_preview.rs | `2f227d4d00ce9a7090f36fc6a4b5f457df9429abf08515e3e037782950496467` | [[1, 72]] |
| crates/adventuresim-character-creator/src/armor_metal.rs | `7c0ab220dff0ffa44df39a73258de057c9b5d7037f694eb2ce32f81d132e5e63` | [[1, 171]] |
| crates/adventuresim-character-creator/src/metal_controls.rs | `58cb430aed05d49c2d1e9782ff773903f6c26ac15c304e523a4e0a2537ec541d` | [[1, 201]] |
| crates/adventuresim-character-creator/src/ornament_controls.rs | `c10940e30bdb0ae77feaca37d5d33b6aac96e7cd7a97a48fc729c69be81aad3a` | [[1, 43]] |
| crates/adventuresim-character-creator/src/fabric_controls.rs | `4f576fdfae6f68499082f3da87dbba83f1e587242edc969720b1982a89fb66b0` | [[1, 71]] |
| crates/adventuresim-character-creator/src/construction_controls.rs | `cc9d26e5f71807cfa63e701e752330432e109c59f612b20544df220c59b5a9bd` | [[1, 93]] |
| crates/adventuresim-character-creator/src/decoration.rs | `7b677abc0f1a167c6b25343e2964d611aa479132a32330ebd2bd8b03fbf2619e` | [[1, 153]] |
| crates/adventuresim-character-creator/src/decoration_controls.rs | `ca0d51e0b1984ca87534afb472882f25280b26fb445232ec0e3e178920e3f3a2` | [[1, 97]] |
| crates/adventuresim-character-creator/src/library_name.rs | `ecd978e94b850604d4345df1494e9536819e19d7a6d3b8921fde3d55bf54eb27` | [[1, 52]] |
| crates/adventuresim-character-creator/src/wardrobe.rs | `41051202a9da2998dcf73b5a1e095088d6838c1a849a66d080631ac8304951b9` | [[1, 154]] |
| crates/adventuresim-character-creator/src/wardrobe_tab.rs | `ec1437075d1383eeae19ee3e2643e4d918de6a2032fe1e1ba83135d5f2353a0f` | [[1, 325]] |
| crates/adventuresim-character-creator/src/drape_preview.rs | `a248f8b0ae468ad6f590396c5f2413900f000b9d86fb41b75225d829b2aa8d87` | [[1, 305]] |
| crates/adventuresim-character-creator/src/underlayer_preview.rs | `c27d902cac7ee61e69322688e8be4404d849c07c982616801ecb7d960f7d0d25` | [[1, 137]] |
| crates/adventuresim-character-creator/src/underlayer_material.rs | `52ebd2ffe14baf52f9e8bdd18ecf23fb26b94b1cf52c655fbc58074cd18471b7` | [[1, 27]] |
| crates/adventuresim-character-creator/src/preview.rs | `b17010a478ba122880bf826707227f55e2833023b46ec14cfc6baceb09237847` | [[1, 177]] |
| crates/adventuresim-character-creator/src/character_export.rs | `6210517f4a5b5ff82c940bfaddb1581c206b3c92630e45a1e733c2ccf5f23bdd` | [[1, 242]] |
| crates/adventuresim-character-creator/src/character_export/draped.rs | `2c0a2e659aa914d0b575343dc1ae1fd7216cfa370462be0bfb1308745105526e` | [[1, 79]] |
| crates/adventuresim-character-creator/src/equipment_export.rs | `dbbb086bd77ecd0153686025543f523cc7787e9a909a030625275c6d124a9b59` | [[1, 342]] |
| crates/adventuresim-character-creator/src/export/shell.rs | `e430c45bc2b8f6b87ab984c3b1684a287e124a4275eb29f8ab86906db79093e2` | [[1, 117]] |
| crates/adventuresim-character-creator/src/export/textures.rs | `5d23e1c4decd680acec00f3d2bf5f5a66beac501291d73165cafe7f9ccd26322` | [[1, 250]] |
| crates/adventuresim-character-creator/src/garment/export.rs | `d4d9bf1aea7c1a69e348644cf11a09056260ce34f97b340357aed60c646c4a5a` | [[1, 30]] |
| crates/adventuresim-character-creator/src/garment/selection.rs | `887b9ace0ef021d409c493c8e8e109c13dbd98a5dd21251da6abecbf44be6ba7` | [[1, 271]] |
| crates/adventuresim-character-creator/src/garment/settled.rs | `3dbafb11ea673b81b72c6d8599bf9c1ac4c55b3e456982e39d16d38faef2e42d` | [[1, 308]] |
| crates/adventuresim-character-creator/src/inventory.rs | `0f820bca06e27b164da010e95dc7d3500a79a6feb0b7be9411ed8a41e88feb18` | [[1, 442]] |
| crates/adventuresim-character-creator/src/inventory/loadout.rs | `8b3ada636d68eaae16cf4ca2bab097cd918be93a52452d34dc3e2648add51691` | [[1, 110]] |
| crates/adventuresim-character-creator/src/inventory/tests.rs | `3c8c66c5b49199d343845589bf635cd68c05dbd740656a1e33bdbc7c0534f4cd` | [[1, 239]] |
| crates/adventuresim-character-creator/src/inventory_article_editor.rs | `bfaf8082b9b0e3aba214dbe23016ae3d83f78b3a66f777ea6e2601ddc5475cf9` | [[1, 199]] |
| crates/adventuresim-character-creator/src/generation.rs | `eb3d54908cc923b9cd761ca5abaf748d4ad9ed9175289a2d3342d0e2d4c6a4a1` | [[1, 93]] |
| crates/adventuresim-character-creator/src/studio_generation.rs | `859437a4b9cd33e71a2fb2c34106ba80bb92ec545d3382c2a363218b91088659` | [[1, 187]] |
| crates/adventuresim-character-creator/src/armory.rs | `74f95648715bc2456ec986093c649779f02d3c773607d24af7ff0ccfc4ca18ef` | [[1, 427]] |
| crates/adventuresim-character-creator/src/armory/exhibit.rs | `b2f4f006cf78814970012ab94d85d178176813c53cd2fbd9768ff2b17349c221` | [[1, 95]] |
| crates/adventuresim-character-creator/src/armory_scene.rs | `65b095038c61727c57b21a007cee9cf6b655b21a64289c90df206b322b623326` | [[1, 288]] |
| crates/adventuresim-character-creator/src/armor_controls.rs | `ffb59abadd6add1be06909169480d7b845f7e90652cbadf9cc360c3fe3d50e33` | [[1, 180]] |
| crates/adventuresim-character-creator/src/parametric_equipment.rs | `d71953e80a22f666f64d7733ae41279fe18700e1f6b0531ee1cfd404f4c95559` | [[1, 478]] |
| crates/adventuresim-character-creator/src/outfit.rs | `4d3281fb80e63eeb57bf3caaec77e63eb33d75c62e6ec5d7053e2fd76ebd95f2` | [[1, 104]] |
| crates/adventuresim-character-creator/src/clothing/error.rs | `3a33f6fd8aaabd68b8be71ea21f472b142fd17f7648bd79551af19b2c8f24297` | [[1, 48]] |
| crates/adventuresim-character-creator/src/chainmail_export_test.rs | `48cd003151326983ff2bb384afe95995e95fe0c18825faa497769ea419241b94` | [[1, 87]] |
| crates/adventuresim-character-creator/src/cli.rs | `75a646db6855544be80e795e8a46aa8467929113159c7c94011e641e00472c9c` | [[1, 70]] |
| crates/adventuresim-character-creator/Cargo.toml | `89cf06ca88fa0b2863f103f8cf6961345e3c5cfc2c52ad8802bbe3f452d70361` | [[1, 83]] |
| crates/fabelgeist-armor/review/museum/henry/trim.json | `e9ebce2edb4db3d4d4a87cce7400ef478390384d601925de284b350f0a078482` | [[1, 58]] |
| crates/fabelgeist-armor/review/museum/nuremberg/trim.json | `dc8bd427433cf3c8ee954b903597248a21b3bfa3332ad90c83bdb5f8af2df1a6` | [[1, 11]] |
| scripts/trim_armor.py | `dfa66cabcaf180fa6a651146b8360822ba544f4aeda021b8effdcdb4db406a2b` | [[1, 182]] |
| scripts/armor_trim_math.py | `c5e606ca6061e877a0405535d0a6580a7c504cdbfc7d2cefc4214035d8c98c85` | [[1, 89]] |
| scripts/finish_equipment.py | `23703961617f28c82c8f40eb04376845293486f723ddb5ce87643b941d6df69c` | [[1, 37]] |
| crates/fabelgeist-armor/src/lib.rs | `b15110d340873bcac7853670061621fefdd69388dec331613c316bf5bbec1bf9` | [[1, 170]] |
| crates/fabelgeist-armor/src/textile_pattern.rs | `eb6c1803c7a95f765e79baf0f41e56337a769930576928064e201147ac404ef2` | [[1, 205]] |
| crates/fabelgeist-armor/src/textile_needlework.rs | `7e208e1ea8b527fa4671cd3e09bfaa29f7155da8701687730ec059f419de1021` | [[1, 140]] |
| crates/adventuresim-character-creator/src/main.rs | `366087f0e3753b4917fc7fb7fe1f5aa1c6bc654dd4fe3ed03d97c7df0f3395e8` | [[180, 410]] |
| crates/adventuresim-character-creator/src/studio_ui.rs | `ab4430b01df6df4b77ee2e29a491531bd3a5782c92fd16b67a9aaba2d239b394` | [[190, 245]] |
| crates/adventuresim-character-creator/src/armory_ui.rs | `dea23f6efe94e9b5439739548675be1b0cfd109a3e153e2a083d7a816db43aeb` | [[230, 290]] |
| crates/fabelgeist-armor/review/museum/README.md | `18d36ba79517398aea316df0b69b8d93a736955c3b97160d86e974806ec0e01d` | [[193, 225]] |
| crates/fabelgeist-color/src/lib.rs | `9586a92ca2dcf19911b96b5fd50818df42af552582a7b0955224294ef69b9013` | [[1, 26]] |
| crates/fabelgeist-color/src/color.rs | `ec95ce41e0de2b14f31c2d8a206461d2ed69f95183e18ae020c7ce956de86c74` | [[1, 80]] |
| crates/fabelgeist-color/src/encoded.rs | `207692aa2840d13ab24b0bd67e5d228df64245ed80345ea91e833ad23ff827a2` | [[1, 73]] |
| crates/adventuresim-character-creator/src/clothing_material.rs | `392c3f5ab383b7eab671eafcfb84d79bf5a013ee8e21b7acb395f22788d5624a` | [[1, 55]] |
| crates/adventuresim-character-creator/src/bare_body.rs | `10911f928ebfe620f5a0e3cfd348c481d7d455a52788cfd1a39e0ff2f02c97ad` | [[1, 69]] |
| crates/adventuresim-character-creator/src/studio_cache.rs | `6d7378554d92fcefc39d39434744bdb50dd98358ce1596f571652a65ce4bf013` | [[1, 192]] |
| crates/fabelgeist-armor/src/trim/tests.rs | `b0a2e919bc35d3820de076231dceb2758c9f5cf07d2670e52618485b6352ef7b` | [[1, 276]] |
| crates/fabelgeist-armor/src/error.rs | `166181e2197cd6959a0f894ef512effae10f1ff508b6aec4108aabdcaee919be` | [[1, 42]] |
| crates/adventuresim-character-creator/src/item_design.rs | `a33bdf3bde70ed5a647c3968c9e9c04dd60402bca585d47a4d2555ce9e3802aa` | [[1, 232]] |
| crates/adventuresim-character-creator/src/export/rigged.rs | `73b6b6c79dad123790c5136a530112273cc766f8602f07360d36ffe5b1bb8cd6` | [[1, 60]] |
| crates/adventuresim-character-creator/src/armor_recipes.rs | `8b2449df30f3da7cfaa1478203f428c4ac9517ff007e97396c7cfc916c11099a` | [[1, 145]] |
| crates/adventuresim-character-creator/src/garment.rs | `3a53a6a40c78d926e2f50031f319b7fb338014633b6c73431b891b85a308c828` | [[1, 160]] |

## Unrefreshed frontier

These paths have hashes only in the JSON ledger. They have not been freshly read
completely for this remaining-family discovery; they are not cleared by their
presence in earlier reports.

- crates/fabelgeist-armor/src/components.rs
- crates/fabelgeist-armor/src/textile.rs
- crates/adventuresim-tactical-client/src/equipment/runtime_equipment/meshes.rs
- crates/adventuresim-character-creator/src/clothing.rs
- crates/adventuresim-character-creator/src/character_morphs.rs
- crates/adventuresim-character-creator/src/cloth_fastener_mesh.rs
- crates/adventuresim-character-creator/src/cloth_hook_mesh.rs
- crates/adventuresim-character-creator/src/cloth_lacing_mesh.rs
- crates/adventuresim-character-creator/src/cloth_closure_controls.rs
- crates/adventuresim-character-creator/src/export/validation.rs
- crates/fabelgeist-color/src/transfer.rs
- crates/adventuresim-procedural-textures/src/palette.rs
- crates/adventuresim-character-creator/src/export.rs
- Cargo.lock
- crates/adventuresim-character-creator/Cargo.lock
- crates/fabelgeist-geometry/Cargo.toml
- crates/fabelgeist-color/Cargo.toml
- AGENTS.md
- crates/adventuresim-character-creator/src/textile_surface.rs
- crates/adventuresim-character-creator/src/character_export/hair.rs
- crates/adventuresim-character-creator/src/export/compact.rs
- crates/adventuresim-character-creator/src/export/assembly.rs
- crates/adventuresim-character-creator/src/export/morphs.rs
- crates/adventuresim-character-creator/src/export/textures/tests.rs
- crates/adventuresim-character-creator/tests/textile_export.rs
-
  crates/adventuresim-tactical-client/src/equipment/runtime_equipment/mesh_tests.rs
- crates/fabelgeist-hair/src/parameters.rs
- crates/fabelgeist-armor/src/textile_seams.rs
- crates/fabelgeist-armor/src/gpu/body.rs
- crates/adventuresim-character-creator/src/device_fit.rs
- crates/adventuresim-character-creator/src/fitted_existing.rs
- crates/adventuresim-character-creator/src/underlayer_armor.rs
- crates/adventuresim-character-creator/src/underlayer_equipment.rs
- crates/adventuresim-character-creator/src/inventory/occupancy.rs
- crates/adventuresim-character-creator/src/garment/pattern.rs
- crates/adventuresim-character-creator/src/garment/stages.rs
- crates/adventuresim-character-creator/src/underlayer.rs
- crates/adventuresim-character-creator/src/studio_overlay.rs
- crates/adventuresim-character-creator/src/equipment_catalog.rs
- crates/adventuresim-core/src/character_proportions.rs
- crates/adventuresim-character-creator/src/review_export.rs

Do not derive `From<String>` for `LibraryName`: that would bypass checked
whitespace admission. Keep the checked `TryFrom` route. Only equivalent
mechanical outbound/projection derives are candidates.
