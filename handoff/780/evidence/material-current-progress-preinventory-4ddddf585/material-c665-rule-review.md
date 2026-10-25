# Material family independent Rule Review

Pinned incoming `c6659f6cfa4610f066c659411ade6636e21bd3b5`, tree
`12f2d808ab9d659999aaec627e02e594897c8beb`, with Root unchanged at
`cac5a7c33a83d1c08a5e55d43d912967edf06814`. The separate complete
thirteen-lens inventory precedes this review. JSON individually records all22
candidate hashes, six reference-only hashes and20 actual recipient accounts,
including exact coverage limits. All candidate bodies and existing fixtures
were read completely. No source was received or repaired.

Whole-family correctness and completeness are not clear. Six actionable groups
remain; none should be silently deferred because some source matches main:

- **MAT-C1, P2: armor_metal.rs:55–60.** The public density helper accepts
  finite geometry and tiny positive finite UV spacing, passes its guard, then
  narrows a finite f64 scale above f32::MAX to infinity. Its in-place writes
  publish Inf/NaN UVs. This is an existing connected operation defect, not a
  c665 regression. Check candidate scale and every output before mutation,
  propagate structured failure through fitted_catalog_item and adapt its
  existing fixture call. Preserve the explicit zero/no-density policy; do not
  introduce an arbitrary epsilon. The bespoke review's operation contract and
  crates/AGENTS.md:31–57 require units and valid replacement behavior.
- **MAT-R1, P2: material.rs:15–30, engraving.rs:25–34/66–76,
  trim/finish.rs:12–15/46–71 and ornament.rs:13–45.** Material lengths,
  ratios, repeat/count/seed roles and the fillet policy remain raw through
  configuration, controls, trim operations and retained recipes. The new
  immutable ValidatedMetal correctly checks terminal native admission, but
  does not complete semantic producer propagation. Reuse matching shared
  Metres/Ratio/PerMetre owners and coordinate remaining bounded/role owners
  through actual consumers, as required by crates/AGENTS.md:28–71. Preserve
  native32 LCG words and serialized sRGB3 colour semantics; Seed64/RGBA4 are
  not automatic substitutes. This is an existing scope migration gap, not a
  demonstrated new invalid native bake.
- **MAT-R2, P3: clothing_material.rs:3, armor_metal.rs:68–87,
  metal_preview.rs:14, material.rs:74–87 and ornament.rs:92–101.** PBR,
  trim result, preview depth, validation ranges and motif projections retain
  positional semantic records. The actual native interfaces do not require
  those tuples. Carry named records with semantic fields through the family
  under crates/AGENTS.md:45–51/66–68. Required framework tuples and named
  mathematical sine/cosine pairs remain valid native boundaries.
- **MAT-R3, P3: material, engraving, map, ornament and trim error owners.**
  Their cohesive fallible APIs repeat the same error parameter without owning
  Result aliases. Introduce one appropriately scoped alias and reuse it in
  children, following crates/AGENTS.md:118–122. Retain explicit trait and
  genuinely distinct error signatures; do not widen carriers or add aliases
  for compatibility.
- **MAT-R4, P3: trim.rs:22–26, decoration.rs:22–45 and
  export/textures.rs:5–6/89.** Modified production owners retain re-exports
  after modules, a struct after an impl, and a test module before production
  items. Move exact item/doc/attribute blocks into mandated order under
  crates/AGENTS.md:90–94. Unrelated export.rs production is exact Root and
  only its fixture operand changes; inherited socket order is recorded
  separately, without an independent rewrite authorization.
- **MAT-R5, P2: inventory.rs:177–192/358, lib.rs recipe validation,
  generation.rs:7 and main.rs:223/363/377.** Structured decoration validation
  is formatted into String in the domain admission chain, then wrapped with
  Error::msg before final presentation. Preserve its typed provenance through
  actual recipe load/save/generation consumers, or explicitly settle that
  boundary contract. This follows crates/AGENTS.md:85–86 and the required
  producer/consumer review. These validators do not perform image I/O; no
  claim is made that native I/O errors traverse this particular barrier.

CCR-D1 has a concrete canonical remedy but remains unresolved in Root. Incoming
MetalImages::new returns anyhow::Result, uses the checked PREVIEW and consumes
size()/into_maps(), preserving actual causes into EquipmentMaps. Root still
has the String/raw-field implementation. Receive the complete matching
material/export/caller dependencies, not a raw overload or forwarding shim.

The new core contracts are sound source increments within those limits. The
checked tile controls supported side and exact native word population. Private
MetalTextures admits matching byte lengths and finite nonnegative depth scale.
Cut and EngravingImage preserve validated recipe/decoded relief association.
Actual filesystem and image decode errors retain std::io::Error/ImageError,
path/source diagnostics and downstream anyhow/native causes.

Signed sampling admission was independently traced against actual WGSL. The
extent proof bounds signed wrap addition and packed image words. Sampling uses
actual checked-size pixel centres, CPU sine/cosine coefficients and each
fenced affine operation; corner extrema bound finite floor coordinates and
bilinear plus-one addresses before native casts. This relies on the existing
host_float arithmetic contract and supplies no hardware/browser verification.
Native flags, packed words, glTF JSON keys and sampler enums/constants are
explicit framework ports, not evidence for exempting application semantics.

No new production unwrap/expect, unsafe or Send/Sync assertion occurs in this
candidate. New error derives retain concrete causes; private admitted maps
have no deserialize bypass. Existing macro/fixture/native representation
policies remain distinct. The Root native MetalGpu/Error whole file is exact
the reference; native Buffer/map/context helpers must be preserved. No
reference-only library or native file may be blindly substituted.

Existing tests retain cases, helpers, assertions and oracles. Metal and
engraving changes are API constructor/accessor/operand adaptations; export.rs
changes one existing fixture size operand. Its production body and inherited
baseline803/function ceilings111/268 remain unchanged. Other production
candidates are below500 raw lines. Final relevant hosted baseline reductions
remain required; no scanner or custom checker clone was used.

Canonical counts/stride, KernelBatch, host-float/native helpers, full producers,
GUI/caller/export/lock composition and the named untraced construction/physical
remainders remain required. Complete all-thirteen waves, independent final
reviews and hosted checks must pass on one final pushed revision. No source
edits, new tests/cases/helpers/assertions/doctests, project checks, scanner,
generator, build, lint, test, runtime, push or CI occurred. This report grants
no compilation, native/browser arithmetic, visual/performance, whole-family,
global or final PR clearance.
