# PR #780 UV/maps authority: sequential all-thirteen discovery

Pinned source HEAD: 59bb96d07af08244582c6767d81e969b820ec3ed.
Root's completed density tranche was committed while this discovery was in
progress; the UV source files are byte-exact across that commit. Current
source fingerprints show no drift. The only written files are this ignored
report and its /tmp JSON companion. No source edit, test, build, formatter,
quality tool, execution probe, new case/helper/assertion or hosted check ran.

Discovery is complete for the explicit UV/maps family. PBR-OUTPUT-2 remains
actionable and unimplemented. This is not repair clearance, whole-material
clearance, whole-export geometry clearance or final same-SHA review. Root is
the sole implementer; wait for Root's repair before reviewing a successor.

## Finding and final direct API contract

PBR-OUTPUT-2 is confirmed from actual source, not the prior report verdict.
RiggedShell.surface currently stores UVs with plate maps; texcoords stores a
second optional UV column. validate accepts a finite vertex-sized surface
column without texcoords. CompactShell selects only texcoords by used
source vertex. Its reconstructed surface still points at original UVs.
Writer falls back to that stale slice after compacting positions/faces.

Concrete representable input: four source vertices, one face [1,2,3],
surface Some(four UVs, nonempty maps), texcoords None. Compacted geometry has
three rows, but emitted UVs have four original rows beginning with vertex
zero. No new executable test or case was made to establish this reasoning.

The intended clean schema closes this mismatch:
- surface: Option<&ShellTextures>, carrying only actual image identity.
- texcoords: Option<&[[f32;2]]>, sole per-vertex chart authority.
- Either map family requires texcoords; every present UV column must be
  finite and exactly positions.len() before compaction/native output.
- Plate normal and metallic-roughness image nonempty checks remain.
- CompactShell remaps the sole UV column exactly once, alongside all other
  selected vertex channels. Its current implementation already does this;
  the whole file may remain byte-identical after the field's type change.
- Writer reads only texcoords and consumes surface directly as map reference.
  No old tuple fallback, primitive adapter, duplicated UV cache or compatibility
  schema remains.

## Exact connected implementation and unchanged dependency graph

Potential source edits are rigged.rs (field/docs), shell.rs (sole-column and
maps receiving), validation.rs (both map-family UV presence/shape admission),
character_export.rs (plate/trim map references), equipment_export.rs (the
additional standalone producer), and export.rs's one existing textured
fixture (move the exact same uvs from tuple to texcoords). compact.rs is a
complete reviewed receiver and proof obligation, not a reason to rewrite a
correct mapping algorithm.

Character path:
GeneratedArmor.texcoords -> character_morphs::rigged_armor sets texcoords ->
catalog_shells attaches cached maps -> component override can clear maps ->
export_rigged_glb validate -> CompactShell select -> Writer TEXCOORD_0.

Trim path:
rigged_armor's trim shells -> catalog_shells explicitly replaces texcoords
with trim.texcoords and attaches matching maps -> same admission/compaction.
Keep that actual chart replacement, not just the maps assignment.

Standalone equipment path:
EquipmentExporter::armor -> fitted_catalog_item -> rigged_armor supplies
armor.texcoords -> current surface tuple assignment at equipment_export.rs
91-94 must become maps-only -> component overrides -> public export.
This additional producer is essential even though it was not in the first
list of named changed files.

Other actual constructors:
- rigged_clothing: no maps/no UVs; remains valid.
- component_materials: clears plate and generic maps for a component override,
  then independently attaches textile maps/finish; it retains valid armor UV.
- DrapedGarment::rigged: chooses actual mail or garment UV and generic maps.
- ExportedHair::shell: uses actual cards.texcoords with atlas maps.
- CompactShell::rigged: copies only maps identity and points at its own UVs.
- Existing fixture constructors in assembly.rs, morphs.rs, textures/tests.rs
  and tests/textile_export.rs already use sole texcoords or no maps.

Every production surface assignment and direct RiggedShell constructor was
accounted. The crate-wide locator has no additional RiggedShell constructor
outside the listed Creator files; main.rs only imports the public export
owner. Search was a locator; each actual UV/maps body was read.

## Preservation and non-goals for this repair

The previous Writer's explicit texcoords precedence is preserved: when old
streams differed, the column actually rendered was texcoords. The removed
tuple column does not introduce an alternate UV set. Both generic and plate
map families can coexist and use the same glTF coordinate set zero; do not
replace them with an exclusive enum or add separate chart copies.

Finite signed and outside-[0,1] UVs remain valid. UV zero is valid. Untextured
shells may omit UVs or retain a valid UV column. Empty shell geometry remains
rejected by its existing admission; zero UV rows cannot match admitted
positive geometry. No normalization, clamping, density change, interpolation,
rounding, new cap or unit reinterpretation belongs in this receiving fix.

Preserve vertex/face order, UV float bits, seam and trim chart correspondence,
skin/morph remap, names/hinges/edges, native byte alignment and accessor kinds,
image bytes/index ordering/cache identity, sampler and image color spaces,
alpha/face policies and material override sequence. Writer still applies
generic maps, then textile finish, then plate maps. ShellTextures pointer
cache identifies images and must not acquire topology/UV authority.

Existing textured-garment fixture at export.rs:1275 uses all four vertices;
moving &uvs to texcoords preserves its complete arrays, metadata and oracles.
Existing assembly fixture independently establishes source [1..] UV remap.
Read these existing cases only; no new case/helper/assertion is allowed.
No additional fixture edits are needed solely because None surface fields
now infer a maps-only reference.

SurfaceUvLayout is a distinct anatomical body-atlas mapping used by socket
lookup and bootstrapping, often with a separate UV-face topology. It must
not be conflated with seam-split per-vertex material texcoords. Reading the
whole export.rs accounts this dependency; it does not clear the socket math,
its physical constants, name schema or external input-admission family.

PBR-OUTPUT-1 (skin color-space output), broad named PBR/color/ratio schema,
plate texture density, image-content admission beyond current PNG ownership,
and larger geometry/physical/generated/native families stay open. This
repair must not silently absorb those changes or claim their clearance.

## Sequential lens ledger

All lenses were run one after another with their individual contracts,
dispositions, dependencies and acceptance below. No overlap substituted for
another pass. Discovery started incomplete; every repair and validation cell
remains not started.

### 1. producer-propagation

Sources: export/rigged.rs:23-45; export/compact.rs:24-69,85-104; export/shell.rs:53-78; character_export.rs:171-202; equipment_export.rs:85-94; character_morphs.rs:185-279; garment/export.rs:6-31; character_export/hair.rs:76-95; export.rs:1275-1280

Contract: Every per-vertex UV is produced with the shell topology, retained once, remapped in selected source-vertex order, then written at TEXCOORD_0. Plate map identity is a separate payload.

Dispositions: PBR-OUTPUT-2 remains actionable. Seven files are directly connected: rigged, compact, shell, validation, character_export, equipment_export, export fixture. CompactShell may remain byte-identical: its existing surface copy becomes maps-only through the type change and its existing texcoords path already remaps correctly. Other constructors already set sole UV or no maps.

Repairs: Change surface to Option<&ShellTextures>; remove tuple-producing map callbacks and Writer fallback; existing compactor already maps texcoords and should retain maps only; migrate same fixture uvs to texcoords. No adapters.

Dependencies: Actual rigged_armor supplies GeneratedArmor.texcoords before catalog assignments; trim still explicitly replaces that sole column; component/material overrides clear maps but retain the chart; mail/hair generic map paths unchanged.

Acceptance: Trace every listed producer through validation, compaction and final Writer with no remaining surface UV field/fallback or tuple. No source changes until complete all13.

Discovery: complete. Repair: not started. Validation: not started.

### 2. existing-type-owners

Sources: export/rigged.rs:23-31; export.rs:66-70,275-478; export/textures.rs:26-62,216-237; export/buffer.rs:32-39; character_export.rs:199-202

Contract: RiggedShell.texcoords already owns exact per-vertex native float2 texture coordinates. ShellTextures already owns plate images only. SurfaceUvLayout owns a different anatomical body-atlas face/UV mapping used by sockets.

Dispositions: Reuse these existing owners and separate their responsibilities. No matching checked generic UV leaf in Mesh/Math/Geometry was established; no new universal Texcoord owner is needed for this direct native exporter array. Body-atlas SurfaceUvLayout cannot substitute for seam-split shell UVs.

Repairs: Remove UV payload from surface rather than inventing another chart type, storing a second selected UV vector, or reusing physical metre coordinates.

Dependencies: The scalar pair reaches f32_bytes and native VEC2 accessor. Texture sampler remains existing SurfaceSampling; do not misapply normalized-ratio bounds or sorting/launch quantities.

Acceptance: One shell column with topology relationship; maps-only ShellTextures reference; no new duplicate owner/dependency, normalization or unit reclassification.

Discovery: complete. Repair: not started. Validation: not started.

### 3. admission-contracts

Sources: export/validation.rs:6-96,99-164; export.rs:590-611; export/compact.rs:24-69; export/shell.rs:12-80; export.rs:1275-1280

Contract: Public mutable RiggedShell literals are admitted at export_rigged_glb before compaction or native writes. If either surface map family is present, its sole texcoords must exist, be finite, and have exactly positions.len() rows. Nontextured UVs are also length/finite checked; untextured shells may omit UVs.

Dispositions: Current surface tuple allows valid finite source UVs but bypasses the only remapped column; explicit and tuple streams may differ while individually valid. Proposed maps-only API closes that representational bypass. Preserve existing nonempty normal_png and metal_roughness_png admission for ShellTextures.

Repairs: Check missing UVs for both map families, then sole column finite/length invariant; retain existing plate-map nonempty-byte checks and existing geometry/skin/morph validation. No eager PNG decoding or new image-content validator in UV repair.

Dependencies: CompactShell::new is private and called only after validate; Writer receives reconstructed compact shells under that path. External skin/socket/PBR admission policies are outside this cell.

Acceptance: Missing/short/long/nonfinite UVs reject before buffer or filesystem output; valid finite signed/outside-unit UVs remain accepted; empty UV with positive geometry rejects; map-free UV-free shell remains accepted. Existing textured fixture moves unchanged source uvs to texcoords.

Discovery: complete. Repair: not started. Validation: not started.

### 4. invariant-operations

Sources: export/compact.rs:24-69,85-104; export/shell.rs:53-59; export/buffer.rs:11-39; character_export.rs:185-202; character_morphs.rs:185-203

Contract: The admitted UV row is selected by the exact same used source-vertex BTreeSet order as positions, normals, skin and morphs; faces are rewritten by its same remap. UV values are copied, not normalized, interpolated, scaled or rounded during export compaction.

Dispositions: Current stale surface UV slice violates that relationship. With maps-only surface the existing texcoords selection is sufficient; no second remapping algorithm is needed. Scalar/source count is native host geometry, unrelated to launch/sort populations.

Repairs: Retain only remapped texcoords in reconstructed shells and actual maps reference; Writer emits that column as little-endian f32 values with VEC2/count equal to compacted geometry.

Dependencies: Face/skin/morph membership admission and primitive ordering remain unchanged. Trim assignment must still replace the actual selected shell texcoords before compaction; component map clearing need not clear harmless UVs.

Acceptance: Source rows [1,2,3] map to UV rows [1,2,3] in output order for every map family; no fallible mutation introduced, no changes to numeric arithmetic/packing/alignment/faces/skin/morphs. No new fixtures or assertions.

Discovery: complete. Repair: not started. Validation: not started.

### 5. aggregate-states

Sources: export/rigged.rs:23-45; export/validation.rs:51-70; export/compact.rs:12-20,46-69,85-104; export/textures.rs:174-237; export.rs:66-70,590-611

Contract: One shell topology and one optional per-vertex chart correlate all texture users; either map family requires the chart. Map payloads and compacted geometry lifetimes are borrowed/owned coherently until synchronous serialization.

Dispositions: Two independently valid finite UV streams currently form an invalid aggregate after compaction. Removing the tuple's UV authority is sufficient; public record admission still belongs to validate, not a new parallel checked-shell schema. Both texture families are independent optional payloads and may coexist on the one chart.

Repairs: surface retains only borrowed maps; texcoords retains sole chart, remapped once; combined UV-presence guard is mandatory. Maps reference may remain source-backed while topology/UV arrays are compact-owned.

Dependencies: CompactShell.source borrows maps/material metadata and compacts per-vertex channels; maps have no vertex shape and need no clone/remap. Distinct SurfaceUvLayout for anatomical sockets must remain separate.

Acceptance: No state can carry an unmapped plate UV stream; any public invalid map-without-chart combination is rejected before native use. Borrowed map identity and all other orthogonal metadata survive reconstruction.

Discovery: complete. Repair: not started. Validation: not started.

### 6. enum-shapes

Sources: export/rigged.rs:23-45; export/textures.rs:9-62,174-237; export/shell.rs:70-78; character_morphs.rs:185-203; garment/export.rs:13-28; character_export/hair.rs:53-61,77-94

Contract: SurfaceAlpha, SurfaceSides, SurfaceSampling and GlbOutput already express independent choices with their payload/control-flow owners. SurfaceTextures and ShellTextures may coexist and have ordered effects; optional UV presence is a data-availability predicate, not an encoded choice between materials.

Dispositions: Do not replace two independent map Options with an exclusive enum: that would reject/change currently supported combinations. Do not create a plate-vs-garment UV tag because all map users consume the same TEXCOORD_0. Existing enums and the maps-only Option are sufficient.

Repairs: Remove only neighboring duplicate UV payload from surface; preserve every current alpha/sides/sampling/packaging variant and material override order.

Dependencies: Default glTF texture coordinate set is zero in actual emitted texture-info objects; no alternate UV set is represented by the current exporter.

Acceptance: Plain, generic-map, plate-map and combined-map shells remain distinguishable and supported; map-free shells may retain UVs. No added bool/tag or removed orthogonal payload.

Discovery: complete. Repair: not started. Validation: not started.

### 7. redundant-authority

Sources: export/rigged.rs:28-31; export/shell.rs:53; export/compact.rs:50-52,88-91; character_export.rs:176-185,201-202; equipment_export.rs:91-94; export/textures.rs:94,216-237

Contract: texcoords is the sole authoritative UV column. ShellTextures identifies reusable image bytes; TextureImages.shell_cache caches image indices by actual maps object and is independent of chart identity.

Dispositions: Current texcoords plus surface.0 are competing current authorities; Writer precedence hides disagreement until compaction. Plate/trim producers duplicate references today. Texture cache is legitimate independent authority over image serialization, not another UV cache. Anatomical socket layout is a separate mapping.

Repairs: Delete surface UV reference and fallback, simplify map-producing callbacks, retain existing sole chart and map pointer cache without new snapshots/defaults.

Dependencies: Trim still updates shell.texcoords from trim.texcoords; plate uses existing armor.texcoords. Generic images are serialized per existing rules; shared byte-content addressing remains untouched.

Acceptance: No current UV fact is stored in both the shell field and a map tuple. Reusing same maps on differently shaped shells is valid because only image indices are cached and each shell writes its own remapped UV accessor.

Discovery: complete. Repair: not started. Validation: not started.

### 8. behavior-preservation

Sources: export.rs:590-611,1236-1384; export/compact.rs:24-69; export/shell.rs:53-78; export/textures.rs:174-237; character_export.rs:171-223; equipment_export.rs:77-98; export/assembly.rs:105-315; export/textures/tests.rs

Contract: Valid source behavior preserves selected source row order, float bits, texture image bytes/index order/cache identity, alpha, sides, sampling, material override precedence and all non-UV geometry channels. Intentional correction is stale surface-only UV output; direct API schema change is authorized without compatibility.

Dispositions: Current explicit texcoords wins when both old streams exist; keep that column. Surface-only old construction migrates the exact same source slice to texcoords. Existing textured-garment fixture uses all four vertices and therefore keeps the same observable arrays/oracles after API adaptation.

Repairs: Only direct tuple removal, UV-presence guard extension, Writer sole-column use and producer/fixture adaptation. Retain ShellTextures image nonempty checks, generic SurfaceTextures behavior, signed/out-of-range finite UVs and texture override order.

Dependencies: No changes to skin/morph/socket/GLB numerical math or PBR values; PBR-OUTPUT-1 remains separate open work. Empty geometry still rejected by existing shell admission; untextured UV-free shells remain supported.

Acceptance: After exact API reversal, executable bodies/fixtures outside the intended UV ownership/guard changes are identical. Existing tests only; add no case/helper/assertion and do not infer GPU/visual/project success from static reads.

Discovery: complete. Repair: not started. Validation: not started.

### 9. derivability

Sources: export/rigged.rs; export/compact.rs; export/textures.rs:9-24,26-62,87-105; garment/export.rs; character_export/hair.rs:76-95; textile_surface.rs:101-125

Contract: The changed representation has no serialization, Default, manual Clone/Copy or error trait implementation to replace. Borrowed shell reconstruction performs coordinated geometry/UV selection; it is semantic construction, not a mechanical derive candidate.

Dispositions: No derivability defect in this UV family. Existing SurfaceTextures Clone and texture-cache Default derives remain appropriate. Texture/enum conversion and glTF JSON mapping are semantic/native boundary code and should not be replaced with broad derives.

Repairs: No extra trait/serde/default implementation; retain existing map reference lifetime and SurfaceTextures cloning semantics.

Dependencies: Current dependencies suffice. Do not introduce a blanket raw-shell Default/Deserialize route that would obscure its runtime admission.

Acceptance: No new manual boilerplate or admission bypass; constructor/ref reconstruction remains explicit for topology relationships and unchanged material payload handling.

Discovery: complete. Repair: not started. Validation: not started.

### 10. magic-strings

Sources: export/rigged.rs; export/compact.rs; export/shell.rs; export/validation.rs; character_export.rs; equipment_export.rs; export.rs:1-1785; export/textures.rs; export/assembly.rs; export/morphs.rs; export/textures/tests.rs; tests/textile_export.rs

Contract: Every string in the directly changing UV path belongs to native glTF JSON/schema, terminal validation/context text, or existing fixture output/name/oracle. TEXCOORD_0 and VEC2 are required native output keys/kinds, not internal UV classifications.

Dispositions: No new string identity or prose-based production classification is needed for maps-only API. Preserve native material/image/sampler keys, mimeType/image paths, authored shell names and current error labels. Socket anatomical domain/prefix, socket math error strings, full GLB and identity metadata text read in export.rs are unchanged adjacent source, not UV-map ownership clearance.

Repairs: Update only now-stale surface field docs; reuse current UV-required diagnostic for both map families if appropriate. Do not invent role strings, material-kind classifier, image-path adapter or compatibility text branch.

Dependencies: Current material/texture effects are selected by existing enums and typed payloads; fixture .contains calls inspect existing diagnostics only and are not new production control flow.

Acceptance: No runtime behavior branches on a new human-facing message or UV/map role string. All native JSON keys, fixture names/paths, image MIME/sampler protocols and oracles outside direct API movement remain unchanged; broader socket/text identity migration stays explicitly outside this repair.

Discovery: complete. Repair: not started. Validation: not started.

### 11. magic-numbers

Sources: export/rigged.rs:28-31; export/compact.rs:24-69; export/shell.rs:20-59,65-67; export/validation.rs:51-70; export/buffer.rs:11-39; export.rs:1-1785; export/textures.rs:160-203,233-235; character_export.rs:171-223

Contract: UV records are native float2 array rows. Zero/positive presence and host row lengths belong to actual buffer/geometry admission; UV values are finite signed texture coordinates with no normalized bounds. The compactor performs selection only; output preserves f32 bits.

Dispositions: Arity 2, triangle arity 3, fixed dimension indices, zero-based remap/accessor indices, f32/component protocol 5126, ARRAY_BUFFER 34962, little-endian bytes and four-byte alignment are concrete native geometry/glTF boundaries. Existing alpha .5, image sampler numeric constants and roughness/color values have distinct owners and remain unchanged. No sort/launch count is involved.

Repairs: No numeric helper/new wrapper/cap/tolerance/normalization. Keep source fixture UV arrays exact while moving their sole field. Continue raw ratio/color and socket-unit migrations separately under their existing broader inventories, not as hidden UV edits.

Dependencies: Geometry validity/finite column proof is established before indexing; unique selected vertices arise from u32 face words, preserving remap representation. Actual output count is the selected UV vector length.

Acceptance: No arithmetic or numeric constant changes; finite outside-unit/negative/zero UVs preserved. All unrelated export/socket/calibration and existing fixture numeric literals remain byte-exact in this repair, apart from moving an existing slice reference.

Discovery: complete. Repair: not started. Validation: not started.

### 12. type-ownership

Sources: export/rigged.rs:23-45; export/textures.rs:26-62; export/compact.rs; export/shell.rs; character_export.rs:171-202; equipment_export.rs:77-94; lib.rs:102-103; main.rs imports

Contract: The offline Creator export module owns RiggedShell's per-vertex chart relationship and its map payloads. GeneratedArmor/HairGeometry/DrapedGarment own source columns; the compactor owns selected column copies; TextureImages owns image serialization/cache.

Dispositions: The duplicate UV reference is misplaced in the plate-map field. Keep the sole chart in RiggedShell.texcoords and maps in ShellTextures. SurfaceUvLayout's anatomical body mapping remains its proper separate socket owner. No move to GPU/Compute/Math or lower shared primitive is required.

Repairs: Direct API owner change in existing modules; update actual Character and Equipment exporter producers and writer/validation/fixture consumers. Root remains sole implementer and protected generated-source functions are unaffected.

Dependencies: All affected API is under existing offline-creator export feature. No Cargo/lock/reexport compatibility change is needed; borrowed ShellTextures type already public through export.

Acceptance: No competing UV schema, compatibility adapter/alias, parent-layer type imported by a lower crate or additional implementation. Every direct constructor and inferred assignment continues to use canonical export owner.

Discovery: complete. Repair: not started. Validation: not started.

### 13. type-granularity

Sources: export/rigged.rs; export/textures.rs:26-62; export/compact.rs; export/validation.rs:51-70; export.rs:66-70

Contract: The useful distinction is per-vertex chart data versus image-map identity, not one UV type per material. Optional chart and two optional independent map families express actual exporter states, with correlated admission at the public export boundary.

Dispositions: Removing the tuple payload simplifies an unnecessary duplicate authority and closes the concrete failure. A universal texture-coordinate framework, per-map copied UV buffer, ordinal enum or extra checked-wrapper adapter would not prevent another demonstrated mix-up in this repair. The anatomical body UV mapping must remain distinct from seam-split shell UVs.

Repairs: Use exactly maps-only surface plus existing sole texcoords column and guard. Do not conflate RGBA colors, physical metre pattern coordinates, normalized atlas coordinates or sampling policy with the direct native chart rows.

Dependencies: Existing source conversions (mail metre chart -> repeat coordinates, armor density/trim source and hair atlas UV) retain their actual source policy. This repair only owns receiving association and remapping.

Acceptance: Single owner and single remap are sufficient for the represented native TEXCOORD_0 protocol, with explicit missing/finite/shape admission; no unsupported extra bound or compatibility schema. Broader material/geometry concepts remain open rather than claimed clear.

Discovery: complete. Repair: not started. Validation: not started.

## Source coverage and fingerprints

The six named target files, including all 1,785 lines of export.rs and its
existing fixtures, were read in full. Additional complete receiving files
and scoped upstream/importer reads are identified below. Hashes identify
whole bytes even where only the affected source responsibility was read.
No read coverage is inferred from a historical pass or a successful search.
The companion JSON records explicit per-file coverage and disposition.
No UV/maps consumer remains unread in the represented receiving graph;
broader responsibilities in partial or unchanged files are not cleared.

| Path | Read coverage / disposition | SHA256 |
|---|---|---|
| crates/adventuresim-character-creator/src/export/rigged.rs | whole; planned changed | b53b6a874c7a23f38f7895c75a03c65c599c21e122d45b1843e5b3efa2b3a25e |
| crates/adventuresim-character-creator/src/export/compact.rs | whole; affected receiver; may remain byte-identical after maps-only type change | 9b57fb4bbcebe5b20113a06d530507253693a37d40f85f1617a38cbccb63fa87 |
| crates/adventuresim-character-creator/src/export/shell.rs | whole; planned changed | 93b0aa654eacc98d66c072b54cb1a1d0444b75e6bdc3cc0ecd7d76325f085bd6 |
| crates/adventuresim-character-creator/src/export/validation.rs | whole; planned changed | 61a7e389db804f2c17d7e619c27bb5bfc88f873f11e00d05097c85b6121518fb |
| crates/adventuresim-character-creator/src/character_export.rs | whole; planned changed | 33f6b05450b7b1eb8e1080aa330b35696e6ee4c57da0bb4d208a18f66ebec963 |
| crates/adventuresim-character-creator/src/equipment_export.rs | whole; additional connected changed producer | 2906c6c5115f7924743595c1808b2337d4b167c27d301436ac22304c75479428 |
| crates/adventuresim-character-creator/src/export.rs | whole 1-1785 including fixtures; planned fixture API change only | 82c083e495d9e3b992abf5ad23d3f99f48acc42da6900cd44ca99405140095de |
| crates/adventuresim-character-creator/src/character_morphs.rs | 175-282 material/rigged producers; other current source read in prior frontier, no verdict reused; unchanged affected | 8f76594ec3871a68df2f8e98740aa629e479b1e6fc7a0ad9a63e4e7f029e30a1 |
| crates/adventuresim-character-creator/src/garment/export.rs | whole; unchanged affected | 0b57ba3d094cdb3ac0ea1e872f7177837ffae8c16be441a1dbcb823d220b26dc |
| crates/adventuresim-character-creator/src/character_export/hair.rs | whole; unchanged affected | bc1d25835f665c6c251087d1262e851bc01da86beda413c4d21b398b90cf594d |
| crates/adventuresim-character-creator/src/export/textures.rs | whole; unchanged affected | 1d5fcb4d3997b5c4137ad31b6fc871cc065f0b8ee3689cb87beb35e64b8e07fd |
| crates/adventuresim-character-creator/src/export/assembly.rs | whole including fixture; unchanged affected | 69228ac01428944c7113f479834ac9179928ab25512d652a741cd799c562b494 |
| crates/adventuresim-character-creator/src/export/morphs.rs | whole including fixture; unchanged affected | 49339af440cea4592fc08969075def3ca3380fe9a495d0b77180049f879ceee3 |
| crates/adventuresim-character-creator/src/export/textures/tests.rs | whole; unchanged existing fixtures | 43e385b5074a6c8884aabe7f27f70c016f9b16b16e19a0020d58cd01090de0b8 |
| crates/adventuresim-character-creator/src/export/buffer.rs | whole; unchanged native receiving boundary | c92b80be46c77268cf276c3686aaedbd70bba5be1e4f7d5eaef7041f865be104 |
| crates/adventuresim-character-creator/tests/textile_export.rs | whole; unchanged existing fixture | 3f0d6674ab64abd4e90c50836ccc7028edb39f616d8758b2c6ff25e8380d9059 |
| crates/adventuresim-character-creator/src/garment_material.rs | 1-181 relevant complete MailSurface/new/pbr admission; fixtures previously read not clearance; unchanged upstream | 39b44062b3a3fb8db93c869721bcd0ede9f01f588cbde35ea6cdf4b9a26377e3 |
| crates/adventuresim-character-creator/src/garment_material/rings.rs | whole; unchanged texture producer | 6905eff437f36dde6d25965152bfde07ab25c57596f1d5068c14fb52fe75a222 |
| crates/adventuresim-character-creator/src/textile_surface.rs | whole from current pinned frontier read; unchanged texture producer | 6eb0a624f247ea8c1ed2ffd6b25f65ece4b8742d30dd44d190914c9e0b399726 |
| crates/adventuresim-character-creator/src/underlayer_material.rs | whole; unchanged texture producer | 52ebd2ffe14baf52f9e8bdd18ecf23fb26b94b1cf52c655fbc58074cd18471b7 |
| crates/adventuresim-character-creator/src/main.rs | imports/module and RiggedShell reexport consumer only; unchanged importer | 566a69fc6cd50bd7d7ea5c0347567c39f2bdc37e9f8004675dc5328e3751e12e |
| crates/adventuresim-character-creator/src/lib.rs | module export feature declaration; unchanged feature owner | e468ea09b75d60d64e6b1d7612bc80f852b125d62c7473f34bfcd2a878f1b2dc |
| crates/adventuresim-character-creator/Cargo.toml | whole current pinned frontier read; unchanged dependency owner | 1346098aa561b83aa6d045c8744f19edc6605674fd0bbf313af4dcfc4ef595e5 |

Final source drift: [].
