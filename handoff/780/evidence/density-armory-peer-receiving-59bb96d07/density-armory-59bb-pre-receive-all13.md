# Density and Armory receiving inventory

Pinned `59bb96d07af08244582c6767d81e969b820ec3ed`, tree
`4cd32ada6fcfa89ff977cd836ee2b0649df73d28`, compared with
`f2c7422fd6a0f995eb90b72cc9c2a5bd12de4d35`. All eight whole candidate files were
freshly read. All manifest and immutable Git identities match. This source-only
inventory precedes Root receipt; unrelated Root generated work is excluded.

The complete shared source-account tables and exact hashes are in the JSON.
Candidate coverage is2032 lines, plus twelve fresh complete immutable reference
sources. No hash or peer verdict substitutes for body coverage.

| Source | Lines | SHA-256 |
| --- | ---: | --- |
| `S1 crates/fabelgeist-armor/src/material/density.rs` | 122 | `94db413bd0eeefd9fc860abb8eee368ee1d11a55cb48cbb4cd7ba2113fd0d665` |
| `S2 crates/fabelgeist-armor/src/material.rs` | 119 | `12debec1a00cb2ff34e5ef09978a4a3c523bde4bfb1926a1dbef86b7decaf6ea` |
| `S3 crates/adventuresim-character-creator/src/armor_metal.rs` | 171 | `00f657217f6dc1b7b5677176fad2153836a7aabc133a1c7b518bba391a607c16` |
| `S4 crates/adventuresim-character-creator/src/parametric_equipment.rs` | 478 | `d71953e80a22f666f64d7733ae41279fe18700e1f6b0531ee1cfd404f4c95559` |
| `S5 crates/adventuresim-character-creator/src/armory/exhibit.rs` | 95 | `b2f4f006cf78814970012ab94d85d178176813c53cd2fbd9768ff2b17349c221` |
| `S6 crates/adventuresim-character-creator/src/armory.rs` | 427 | `74f95648715bc2456ec986093c649779f02d3c773607d24af7ff0ccfc4ca18ef` |
| `S7 crates/adventuresim-character-creator/src/armory_scene.rs` | 288 | `ff67d80dde117a3e4dd23e394d0a5bc6efe5049ca032c0e5fc0a30eaaeec3429` |
| `S8 crates/adventuresim-character-creator/src/armory_ui.rs` | 332 | `dea23f6efe94e9b5439739548675be1b0cfd109a3e153e2a083d7a816db43aeb` |

Each lens below refers to the shared source table and its actual caller closure.
DG1 is atomic density admission/mutation, DG2 retained fitting/preview state and
causes, DG3 canonical application/dependency integration. Acceptance and
dependency records are explicit in the JSON.

- **producer-propagation** (S1,S3,S4,S5-S8; closure equipment_export,
  studio_cache, character_export, studio_generation): Sole production
  fitted_catalog_item calls the owning method with ?. fit/selected, export
  armor, studio cache and Armory workers propagate the actual error; only final
  Studio/UI status formats it. Correct increment; broader material and
  application producers remain required. Connected repairs: DG1,DG2,DG3.

- **existing-type-owners** (S1,S2; design.rs, units.rs, Armor lib/Cargo):
  GeneratedArmor owns mutation, Metal owns TILES_PER_METRE, Math Ratio owns the
  dimensionless resulting multiplier. Host arrays and u32 indices are the actual
  numerical/collection ports, not GPU launch quantities. Reuse matches actual
  roles; no duplicate generic mesh/device/count owner introduced. Connected
  repairs: DG1,DG3.

- **admission-contracts** (S1:57-83,85-113; S4:94-112): Public mutable
  GeneratedArmor is re-admitted at every density operation: complete triples,
  paired finite columns including unused rows, representable and present
  indices. Only inputs read by this operation are certified. All density
  admission paths funnel through the method; no serde/default bypass added.
  Connected repairs: DG1.

- **invariant-operations** (S1:44-53,85-122): Every candidate output is checked
  under an immutable borrow before a second replacement pass under exclusive
  access. f32 subtraction, f64 square/sum/root and accumulation, ratio narrowing
  then tile multiplication preserve old order. MAT-C1 source counterexample
  closed; no epsilon, arbitrary physical floor or clamping added. Connected
  repairs: DG1.

- **aggregate-states** (S1 DensityInput; S5; S6:295-330; catalog selection
  closure): Private borrowed DensityInput ties triangle membership to the exact
  columns. ExhibitFit owns its successful mesh or error; preview_error is
  independent. wearable guarantees a nonempty retained nonempty-surface
  placement; fit_exhibits maps exactly that placement population. Actual
  constructors/consumers preserve relations; this is no universal
  mesh/skin/morph validity certificate. Connected repairs: DG1,DG2.

- **enum-shapes** (S5:29-34,72-95; S6/S7/S8): Pending, Fitted(GeneratedArmor),
  Failed(anyhow::Error) replace Err("not fitted"). Existing view/frame/region
  enums carry closed roles; steel/stale/visibility are ordinary predicates or
  actual Bevy adapters. New state shape resolves fake-pending and fit-overwrite
  defects; no independent state booleans added. Connected repairs: DG2.

- **redundant-authority** (S1,S3,S5-S7): Old free density helper is removed,
  without forwarding alias. No stored scale/second UV authority. finished
  records the attempted finish; preview_error is replaced by each actual attempt
  and cannot overwrite fitted geometry. Successful fit remains reusable after
  preview failure. Existing retry-on-finish-change/refit policy retained.
  Connected repairs: DG1,DG2.

- **behavior-preservation** (S1-S8; six independent source comparisons in shared
  proof): Empty triangles or constant finite chart leave UVs unchanged. Zero
  surface with positive chart-edge length retains zero scaling. Invalid input
  now fails before mutation. Original edge weighting and output order stay
  exact; UI intentionally stops labeling Pending as failure. Source proof only.
  No numerical/device/artistic runtime validation performed. Connected repairs:
  DG1,DG2.

- **derivability** (S1,S5,S6; Cargo closure): Density error derives thiserror
  and retains TryFromIntError source. No Clone shim for anyhow errors, unsafe
  trait assertion, new derive deserialize, unchecked Default certificate or
  duplicate wrapper. Existing parent derives unchanged; no new
  dependency/feature/lock edit in this packet. Connected repairs: DG1,DG2.

- **magic-strings** (S1,S5-S8; item_design/catalog closure): Density
  classifications are typed. Placement IDs are actual open catalogue keys;
  contextual messages annotate causes. UI labels/status are terminal
  presentation. Missing-parametric-design is still only a prose root carried in
  Failed, recorded as application admission debt below. No prose branching
  introduced, but not all carried application error kinds are typed yet.
  Connected repairs: DG2,DG3.

- **magic-numbers** (S1,S2,S4,S6-S8; units/design closure): Triangle width3,
  vector dimensions2/3, zero presence and actual index/length diagnostics are
  intrinsic native shape/collection roles. Native geometry arithmetic is
  documented in metres/UV frame; tile frequency uses existing owner. Existing
  material/scene lengths, PBR/tuples and UI policy remain broader MAT/type work.
  No new density tuning literal or u32 host-array cap; do not misapply launch
  counts to CPU geometry. Connected repairs: DG1,DG3.

- **type-ownership** (S1,S2,S3,S5; Armor lib and Creator callers): Focused
  material/density owns the operation/error; armory/exhibit owns retained
  fitting outcomes. Creator no longer implements Armor mutation. Reexports
  precede modules in the changed density/Armory owners. Native Root bodies and
  broad libs must be composed, not overwritten; modified-owner residual order
  debts remain required. Connected repairs: DG1,DG2,DG3.

- **type-granularity** (S1,S5; design and canonical units closure): DensityInput
  is an operation-scoped borrowed aggregate, not a cloned universal
  GeneratedArmor validator. Ratio retains exact numerical multiplier; error host
  usize lengths and native u32 index report actual adapter values. Failed
  retains original error rather than placement/message tuple. No overbroad
  skin/normal/morph policy or speculative device/count owner. Existing other
  semantic tuples are not cleared. Connected repairs: DG1,DG2,DG3.

Required dependencies remain:

- MAT-R1..R5 material semantic units, named records, Result aliases,
  changed-owner order and typed application admission remain required; MAT-C1
  alone is repaired here.
- Complete unreceived material22+6, async/native and canonical
  source/physical/count/stride/caller/lock/export graph remains required.
  Selective composition must preserve Root native MetalGpu/Buffer/context and
  current generated work.
- Application admission error ownership, existing StudioCache/ItemDesign
  production expects and modified-owner item order need coordinated resolution
  before final whole-family Rule clearance.
- Same final pushed SHA must pass complete global all13, independent
  Code/Rule/Comment and required hosted checks/baseline reductions.

No source or fixture was changed. No project checker, scanner, generator, build,
lint, test, runtime or hosted CI was run. This is complete bounded discovery,
not complete implementation or final clearance.
