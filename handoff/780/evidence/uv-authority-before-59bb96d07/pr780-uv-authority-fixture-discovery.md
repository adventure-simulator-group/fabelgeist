# Sole UV authority: existing fixture and receiver discovery

Discovery for PBR-OUTPUT-2 before repair. **No implementation, broader PBR or
global clearance.** The completed density/Armory eight-file review is preserved
as a separate verdict. No source edits, tests/cases/helpers/assertions/doctests
were added or changed. No tests, checks, probes, builds or hosted jobs were run.
Root remains the sole implementer.

Worktree:
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`.
Exact current hashes, read extents and constructor accounts are in
`/tmp/pr780-uv-authority-fixture-discovery.json`. Repository-wide Rust occurrence
frontier: `/tmp/pr780-uv-authority-all-rust-occurrences.txt`; receiving field
frontier: `/tmp/pr780-uv-authority-boundary-occurrences.txt`. Searches locate
sources; the conclusions below come from fresh actual source reads.

## Current authority mismatch and final API obligations

`export/rigged.rs:27-31` independently retains surface `(UVs,ShellTextures)` and
optional texcoords. The writer (`export/shell.rs:53`) prefers texcoords and
falls back to the surface tuple. CompactShell selects only texcoords through
its sorted used-vertex set and carries the original surface tuple unchanged.
Thus the module claim that every channel receives identical remapping is false
for the fallback surface UV path. Validation currently admits both independent
UV arrays against original positions, not their later compaction association.

The proposed final shape removes that second authority:

- `surface: Option<&ShellTextures>` carries baked maps only.
- `texcoords` is the sole per-source-vertex UV column, optional for untextured
  shells and required when either map payload is present.
- Validate that sole UV column against all source positions and finite values
  before compaction; require nonempty plate normal/metallic-roughness payloads
  when surface is present, as the current validator does.
- Compact the sole UVs by the same BTreeSet used order as positions, normals,
  selected skin, weights and morph deltas. The returned rigged shell borrows
  only those compacted UVs. Preserve exact face/plate-edge remapping and order.
- Writer consumes texcoords directly, with no fallback or duplicated selection.
  Producers choose the required chart explicitly; the map payload cannot
  implicitly supply or override a vertex column.

`export_rigged_glb` currently validates source shells, compacts them, obtains
compacted morph targets, then calls Writer/morph/assembly on compacted shells.
This ordering must remain. No default/generated UV fallback, shadow UV cache,
compatibility tuple API or temporary primitive forwarding adapter is justified.

The former writer's explicit-texcoords precedence must be reflected in producer
choices. Removing maps from a tuple does not authorize replacing an already
chosen explicit chart with the tuple's other UV array. In ordinary plate/trim
producers the two references currently agree; the sole-UV API makes that
association explicit instead of retaining conflict-capable state.

## All direct fixture constructor sites

Fresh repository-wide `RiggedShell` occurrences identify **ten direct fixture
literal sites** in five files. An array-map site creates multiple shell values;
this count is source sites, not extra test cases. All ten sites and every
surrounding existing assertion/helper were read. Five other literal sites are
production producers/remapping, listed later. No additional fixture literal
site was found in the repository Rust occurrence frontier; this is not general
alias/type-inference proof for unrelated external repositories.

| Fixture source / literal | Current maps / UVs | Required same-scenario disposition |
|---|---|---|
| export.rs:1151 clothing primitive | No maps; texcoords=None | Retain untextured shell and inherited body skin route. |
| export.rs:1275 independent textured garment | surface=Some((&uvs,&textures)); texcoords=None | Move exactly `uvs` to texcoords=Some(&uvs); surface becomes Some(&textures). No additional construction/helper/scenario. |
| export.rs:1402 shell-only belt | No maps; texcoords=None | Retain shell-only topology, skin and socket expectations. |
| export.rs:1547 independent armor/morph | No maps; texcoords=None | Retain independent skin and named morph API. |
| export.rs:1695 fitted socket belt | No maps; texcoords=None | Keep the separate anatomical SurfaceUvLayout; it is not a rendered UV fallback. |
| export.rs:1753 invalid open belt | No maps; texcoords=None | Preserve existing unresolved socket rejection scenario. |
| export/assembly.rs:162 array-map for three named components | SurfaceTextures; explicit texcoords | Keep sole source chart and exact nontrivial compaction oracle. |
| export/morphs.rs:135 mutable test shell | No maps; texcoords=None | Preserve existing three-step channel/NaN validation observations. |
| export/textures/tests.rs:40 existing export helper | SurfaceTextures; explicit texcoords | Preserve existing helper and both calling texture-export scenarios. |
| tests/textile_export.rs:36 sheer textile | SurfaceTextures+textile finish; explicit texcoords | Preserve opacity/pattern/sides/embedded-image observations. |

Only the independent textured-garment literal currently exercises the removed
surface-only UV representation. Existing None literals do not require a new UV
field value simply because the surface field type changes.

## Exact fixture obligations and current coverage limits

### Independent textured garment — export.rs:1245-1383

Read the complete test, including its existing baked Metal::default maps,
original skeleton helper and all binary/glTF assertions. Four garment vertices,
three faces using all four vertices, independent normals/skin, four original
`uvs` rows and exactly the same baked ShellTextures remain the fixture.

Preserve all current assertions: two meshes with one primitive each; material
name Tunic; encoded red.1 decoded to approximately.0100228 within1e-6;
material index1; independent NORMAL and JOINTS_0 accessors; exactly five
attributes (position,normal,joints,weights,UV), each accessor count4; index
count9; exactly two embedded PNG images; normal and metallic-roughness texture
roles; MIME image/png; and each image's MetalTextureSize::PREVIEW dimensions.
Do not replace the maps with a stub or remove GPU-backed baking to simplify the
API change. No new test or alternate case is authorized.

This existing test uses every vertex. It checks UV accessor count, not exact UV
values or the fallback path with unused vertices. Preserve that honest limit;
passing it alone would not establish the repaired compaction relationship.

### Compacted component fixture — export/assembly.rs:106-EOF

Read the entire file, including the production writer and complete existing
fixture. The three shell instances are skull/bevor/visor, face `[1,2,3]`, and
four source UV rows `[0,0],[.2,.3],[.4,.6],[.8,.9]`. It supplies SurfaceTextures
and texcoords directly; this is the existing nontrivial UV compaction oracle.

Preserve the exact `read_tex_coords(0) == texcoords[1..]` assertion. Preserve
positions[1..], compact selected skin, every morph deltas[1..], ordered target
names/zero weights, three mesh/node names, independent hinges and the physical
plate-edge remapping that discards the edge containing unused vertex0.

Texture assertions remain nine exact embedded PNGs (three per shell), correct
MIME and byte payload, Mask alpha, normal map, distinct color/normal/AO image
roles, AO strength1, and original scene/node skin/hinge metadata. This fixture
does not set ShellTextures surface; do not claim its exact UV oracle separately
covers every plate-map route. The shared compactor must supply the source proof.

### Remaining export.rs fixtures

The clothing test preserves separate materialed skinned primitives, distinct
normals/skin, red decode, ordered body/shell morph names, initial zero weights
and original compacted counts4(body)/3(shell) with original position deltas. The
same existing read_document and attachment_test_skeleton helpers were read.

The shell-only equipment fixture preserves one primitive, two skin joints,
pelvis-local attachment socket ID/domain/UV/transform, correct skeleton child
membership, non-root scene membership and no automatic rig attachments.
Independent skinned armor preserves its named forearm_width target, zero weights,
independent joint accessor and successful glTF parsing. The fitted socket fixture
retains its separate anatomical UV layout, selected hip anchor, tangent direction,
bootstrap observation and exact error substring for the existing open-shell
scenario. Rendered texcoords authority must not absorb or rewrite that physical
anatomical correspondence API.

### Morph validation fixture — export/morphs.rs:110-EOF

Read the entire source. The existing mutable shell starts with no morph targets:
channel validation fails; assigning the existing body targets succeeds;
assigning the existing nonfinite position delta fails. Surface and texcoords
both remain None. Do not change this into a UV validation case, add an assertion
or introduce a second test helper.

### Texture helper and sheer textile fixtures

Freshly re-read `export/textures/tests.rs` and `tests/textile_export.rs` in full;
the previous report supplied a frontier, not clearance.

The existing texture helper builds an explicit-UV shell. Preserve the normal-only
scenario's encoded color factors, metallic.8/roughness.2, one normal map,
Repeat-S/Clamp-T and sibling PNG. Preserve the shared maps scenario's three exact
PNG roles and bytes, stable content-addressed siblings, Mask, mesh-only GLB,
five directory entries and existing corrupted-shared-file rejection. The helper
and assertions need no new UV scenarios.

The textile fixture preserves Blend opacity, single-sided policy, white base
factor after texture application, embedded pixels `[240,235,220,51]` in sheer
cloth and `[190,150,55,255]` in the opaque pattern. Its sole explicit UVs already
match its rendered positions. No extra transfer, alpha multiplication or plate
maps should be introduced.

## Actual texture precedence to preserve

Writer creates scalar PBR factors, then applies:

1. **SurfaceTextures**, if present: sets sidedness, optional color image and white
   base factor, normal image, optional AO strength1, alpha mode/cutoff and each
   map's addressing policy. No base-color PNG leaves the existing scalar factor.
2. **TextileFinish**, if present: sets KHR_materials_specular and records the
   required extension. This is not another UV column.
3. **ShellTextures**, if present: applies plate normal and metallic-roughness
   images last, forcing roughnessFactor1. It preserves earlier color texture,
   color factor, AO, alpha/sidedness and specular extension. Its own map sampler
   is Tile. The pointer cache reuses map texture indices for the same map object.

One UV channel serves the maps. Changing `surface` to maps-only must not silently
make SurfaceTextures and ShellTextures mutually exclusive, reverse their order,
change sampler choice, duplicate all image bytes, drop AO, or replace alpha/sides
with default plate policy. The current API exposes both payloads and the writer's
order is observable; the proposed UV repair is not a texture-precedence redesign.

The existing fixtures do not directly combine both map payloads. Their separate
oracles and an exact source comparison of texture application order must carry
that acceptance obligation without adding a case.

## Production/API/docs receiving obligations

Freshly read the receiving methods and docs, with exact partial extents in JSON:

- `character_morphs::rigged_armor` sets the metric armor UV column on every plate
  and band shell; rigged_clothing leaves it absent for untextured body topology.
- `character_export::catalog_shells` attaches maps to those existing plate UVs;
  trim explicitly replaces the sole column with trim.texcoords and attaches trim
  maps. Preserve these explicit chart choices.
- `equipment_export` uses `rigged_armor`'s UVs and attaches catalog metal maps.
- Component overrides clear plate maps and ordinary textures before assigning
  their independent appearance; retain that actual override behavior. Broader
  component appearance reconciliation is outside this UV discovery.
- `DrapedGarment::rigged` selects its chainmail chart when present, otherwise its
  own cloth chart. The selected field remains the only rendered UV authority.
- ExportedHair uses card texcoords and SurfaceTextures; no plate-map fallback.
- CompactShell::rigged carries map references but borrows UVs only from its own
  selected column; no retained source-array address may be reused as output UVs.

Update `RiggedShell.surface` Rustdoc from "maps and the UVs" to maps-only, and
state on texcoords that it is the sole per-vertex column, required by either
map payload. Retain exact seam/cut-edge wording because producers still provide
those columns. Compact module documentation can accurately claim identical
remapping only after every emitted UV path consumes that selected column.
Writer and validation docs must not imply a fallback still exists. No new
example or documentation test is needed to express these field contracts.

Keep existing skin/body fallback, physical plate edges, hinge, morph order,
encoded color transfer and PNG ownership contracts separate. A UV column is not
a geometry index, physical SurfaceUvLayout or map sampler. Native glTF buffer
arrays remain the actual framework projection boundary.

## Acceptance and remaining limits

Record the above fixture, producer, validation, compaction, writer and texture
application roles in the complete pre-edit all-thirteen ledger. Repair the whole
connected surface/texcoords family before pushing or asking for hosted checks.
Adapt only the required existing constructor/API uses; preserve every existing
fixture scenario, helper and assertion. Prove source equivalence for unrelated
numeric/image/native behavior and the retained texture order. Then independently
review final current bytes and use required hosted checks as authorized by Root.

This discovery read all ten direct fixture literal sites and their complete
surrounding fixture bodies/helpers. It does not clear earlier export.rs socket
algorithms or its entire pre1030 test suite; only their relevant helpers and
rendered export pipeline were read. Production character_morphs, character_export
and equipment_export reads are bounded to the actual receiving methods, not
whole-graph review. Public malformed UV admission, source lifetime/compaction
proof and full post-repair reviews remain required. No new malformed-input test
or additional oracle was invented here, and no old fixture verdict supplies
runtime success.

## Exact current source snapshot

HEAD at accounting: `59bb96d07af08244582c6767d81e969b820ec3ed`.

| Source | Read extent | SHA-256 |
|---|---|---|
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/tests/textile_export.rs` | whole current source | `3f0d6674ab64abd4e90c50836ccc7028edb39f616d8758b2c6ff25e8380d9059` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/export/textures/tests.rs` | whole current source | `43e385b5074a6c8884aabe7f27f70c016f9b16b16e19a0020d58cd01090de0b8` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/export/morphs.rs` | whole current source | `49339af440cea4592fc08969075def3ca3380fe9a495d0b77180049f879ceee3` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/export/assembly.rs` | whole current source | `69228ac01428944c7113f479834ac9179928ab25512d652a741cd799c562b494` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/export/compact.rs` | whole current source | `9b57fb4bbcebe5b20113a06d530507253693a37d40f85f1617a38cbccb63fa87` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/export/validation.rs` | whole current source | `61a7e389db804f2c17d7e619c27bb5bfc88f873f11e00d05097c85b6121518fb` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/export/shell.rs` | whole current source | `93b0aa654eacc98d66c072b54cb1a1d0444b75e6bdc3cc0ecd7d76325f085bd6` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/export/rigged.rs` | whole current source | `b53b6a874c7a23f38f7895c75a03c65c599c21e122d45b1843e5b3efa2b3a25e` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/export/textures.rs` | whole current source | `1d5fcb4d3997b5c4137ad31b6fc871cc065f0b8ee3689cb87beb35e64b8e07fd` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/garment/export.rs` | whole current source | `0b57ba3d094cdb3ac0ea1e872f7177837ffae8c16be441a1dbcb823d220b26dc` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/character_export/hair.rs` | whole current source | `bc1d25835f665c6c251087d1262e851bc01da86beda413c4d21b398b90cf594d` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/export.rs` | [[1, 95], [585, 925], [1030, 1785]] | `82c083e495d9e3b992abf5ad23d3f99f48acc42da6900cd44ca99405140095de` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/character_morphs.rs` | [[180, 389]] | `8f76594ec3871a68df2f8e98740aa629e479b1e6fc7a0ad9a63e4e7f029e30a1` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/character_export.rs` | [[145, 225]] | `33f6b05450b7b1eb8e1080aa330b35696e6ee4c57da0bb4d208a18f66ebec963` |
| `/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator/crates/adventuresim-character-creator/src/equipment_export.rs` | [[65, 105]] | `2906c6c5115f7924743595c1808b2337d4b167c27d301436ac22304c75479428` |
