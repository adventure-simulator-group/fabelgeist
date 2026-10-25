# Independent Rule Review: Creator strips and shared mesh topology

Snapshot: HEAD `fbab0a1c55b7e1421ec2dbe609b6df539833171c` plus the working
files fingerprinted below. Verdict: **no unresolved actionable finding within the reviewed Creator
strip/topology receiving scope** after the retained-distance and preview-order
corrections below. Broader producer/dependency and global closure remain open;
this is not a final whole-change clearance.
This is source review, not compiler or hosted-check acceptance.

## Resolved item-order finding and corrective receiving review

The preview item-order finding is resolved in the current source. Imports and
re-exports precede production mod declarations; FormChoice and StrandsDrawn,
with their original documentation, precede impl BodyHead; the attached
cfg(test) render_test declaration is last. Comparing the whole file with the
ignored before-correction snapshot after removing only those moved items
established that all remaining source is exact except blank-line separators.
No fixture or function body changed as part of this repair.

Reread all four corrective files (strips.rs, hair/geometry.rs,
hair_preview/assets.rs and hair_preview.rs) and the selected review-correction
preinventory. The card selection now consumes canonical
CardLevelCount::select. Its count is derived from this exact immutable GpuLods
card collection; select returns None only for an empty collection, otherwise
min(requested,last). Consequently the immediate indexed read is proven in
bounds and preserves the old get-or-last outcome without duplicating selection
policy or inventing a missing-element result. Strand ordinals remain a
distinct family and keep their existing absent-level result.

The corrected comments accurately state that segment length is an estimate
target under a subdivision cap, that minimum ribbon width precedes taper, and
that an oversized card ordinal chooses the coarsest prepared cut. They impose
no numerical or physical policy change. No additional actionable finding was
found in this corrective receiving review.

## Resolved during review

The initial working source left `PlacedHair.longest: f32` while its
GroomedHair producer and StripSource recipient used Metres, creating two
source-demonstrable argument/field mismatches and an unfinished migration.
The implementation owner corrected that field. Fresh source inspection now
finds both retained fields typed Metres and direct typed propagation through
prepare -> PlacedHair -> strip_source -> numerical subdivision. The final
groom.rs hash here is
`d6c46f6ef0b1f037cc45648c2baac76b9e383aad331cf1c3757f1e1150702b43`.
This resolved disposition applies to that actual successor source.

## Coverage and contracts

Read the complete changed Creator strip source, both new strip modules,
geometry/groom, preview/assets/controls/export, the complete existing render
fixture, Studio::new and the complete longest producer body. Read the generic
mesh strip owner, mesh count/GPU exports and manifest, CardStripLayout and the
changed CardMesh build, and the actual native layout/card/atlas, logical-prefix,
readback and Bevy receiving interfaces. Read both selected preinventory ledger
entries and current repository/crate rules. Review history and searches were
not used as clearance.

| Family | Source-backed trace and guarantees |
| --- | --- |
| Strip topology | MeshStripCount and checked MeshStripRows -> checked MeshStripLayout -> derived vertices/triangles/corners and strip-major faces -> CardStripLayout/CardMesh and Creator StripSource -> output buffers and Bevy/indexed export |
| Subdivision | typed native strand/card source layout -> at least two rows -> numerical longest-length policy bounded to 1..8 or 1..16 pieces -> NonZeroU32 -> checked row subdivision -> checked complete mesh extent before allocation or launch |
| Source/device | sole PlacedHair::strip_source uses its own body, retained actual native device and Metres longest; cards/ribbons verify actual source and context devices, ribbons admit drawn population as a source prefix; private prepare callers produce body/lods on the same context |
| Output state | private GpuGeometry retains one admitted layout, allocating device and four attributes; StripBuffers allocates every attribute from the same canonical vertex-byte extent and paired lift extent; semantic layout remains authoritative for zero populations |
| Logical read | GpuGeometry::read rejects a foreign device, requests each exact logical vertex-byte prefix, then calls canonical Buffer::read; zero logical geometry returns empty typed arrays rather than physical sentinel records; faces derive from the retained layout |
| Submission | StripSource::build records through canonical folded InvocationCount conversion from MeshStripCount and propagates KernelBatch::submit failure before returning geometry; no primitive compatibility overload or parallel launch implementation |
| Forms/UI | GpuLods private records/layout accessors -> typed distinct strand/card ordinals and Form -> typed StrandCount/PointsPerStrand/CardCount -> preview metadata and controls; Ratio crosses only the actual slider/numerical boundary; fallible budget/row admission propagates to UI status |
| Atlas | canonical CardAtlas/AtlasMip dimensions and bytes -> actual mip metadata -> Creator AtlasLevel presentation/native image bridge -> Bevy image and PNG export; no independently halved width/height authority |
| Native packing | supplied source uniforms and vec4 attribute ABI -> unchanged four shader strings -> shared row topology; Metres/Ratio become native scalars only in subdivision arithmetic or parameter packing |

The shared mesh owner is appropriately below hair/Creator: material-neutral
strip rows/counts and indexed winding live in MeshStripLayout, while CardCount,
CardSegments and strand cardinality keep their physical roles. CardStripLayout
retains the generic topology rather than a second layout-plus-segment source
of truth. Generic triangles remain distinct from generated card strips. The
new conversion impls reside in the existing mesh GPU quantity module and use
canonical InvocationCount and PassParameter receivers.

The admitted mesh bounds justify shader intermediates for nonempty strips:
rows/segments/vertices/corners are checked before allocation; corner bounds also
bound the row-plus-spread arithmetic in the fixed shader. Empty populations
launch no strip work. Native input card vertices retain the producer's checked
four-vec4 writer bound. The generic face iterator preserves the original
[a,c,b], [b,c,d] order for every segment and strip.

New production modules keep imports, module declarations, constants/types and
impls in governing order. Shared mesh/hair topology edits preserve canonical
exports and feature guards. The preview ordering finding is resolved at the current fingerprints. Unchanged unrelated module ordering was not
expanded into an unsolicited repository-wide cleanup.

## Manifests, fixtures and preservation

Both complete parsed lock graphs were compared with HEAD. Only two existing
package records change: Creator gains fabelgeist-mesh and the already resolved
wgpu29 dependency; fabelgeist-mesh gains already resolved thiserror2. There is
no added package/version/source/checksum churn. Creator's separate lock graph
correctly qualifies wgpu 29.0.4 because it also contains another wgpu version;
the root graph has the unambiguous wgpu edge. Mesh's default gpu feature
provides the count-to-byte/launch conversion methods Creator consumes. Native
Device identity uses the actual direct wgpu29 dependency, not a fake brand,
re-export shim or alternate implementation.

Source comparisons established all four moved WGSL string values (BINDINGS,
CARD_ROWS, RIBBON_ROWS, WRAP) byte-identical to their complete HEAD originals.
The entire existing render_test.rs is byte-identical after reversing only the
two typed ordinal formatting substitutions. Its case, helpers, inputs,
render paths, capture ordering and assertions are preserved. No tests, cases,
test helpers, assertions or new shader instructions were added. The production
geometry module extraction has actual resource/layout responsibility; it is
not a pass-through module.

The source policy still clamps authored piece counts, shares and ribbon budget
with its existing numerical semantics. Correct zero population handling
removes the former phantom strand rather than inventing a new physical rule.
Known broader raw physical settings and native bake/grid migrations remain
outside this narrow receiving verdict; no claim is made that they are complete.

## Limits

No source edit, project check, test, build, scanner or hosted CI was performed
by this reviewer. The scoped preview item-order finding has been repaired and source-rechecked.
The already reported native GpuLods grid-count/InvocationCount mismatch and
larger Grid/native-bake/physical migration families remain open. This review
does not grant their dependency closure or the final complete thirteen-lens,
independent-review and required hosted-check clearance on one pushed SHA.
No result here is runtime, image, geometry-fit or GPU-health acceptance.

## Current scope fingerprints

- `crates/adventuresim-character-creator/src/hair/gpu/strips.rs`: `818294f1347dd1b4d5bcd5535dae5c0cfe30b75dd1d4380091bf98a06e906b6b`
- `crates/adventuresim-character-creator/src/hair/gpu/strips/geometry.rs`: `57f15b94c1f1c08bfcc0e214f95fd19640c418ad76f96d698661717076bcd6e4`
- `crates/adventuresim-character-creator/src/hair/gpu/strips/wgsl.rs`: `525528762142f6b0e781d6f29ee1b58f66055b6e41873a86c6da96ba14b15e10`
- `crates/adventuresim-character-creator/src/hair/geometry.rs`: `ba9b0696088257a6f3d984edf137b59435823ff73ba1f176a3fa82d96a3915fe`
- `crates/adventuresim-character-creator/src/hair/groom.rs`: `d6c46f6ef0b1f037cc45648c2baac76b9e383aad331cf1c3757f1e1150702b43`
- `crates/adventuresim-character-creator/src/hair/gpu/bake/mod.rs`: `70a93715a6a926de40b3f71c676839e81df669ff56f41dcbbfa1486560603466`
- `crates/adventuresim-character-creator/src/hair_preview.rs`: `83895f2f6e29bda031bc1764ed1d51cc133051198e963ead3eb0ee728138ffff`
- `crates/adventuresim-character-creator/src/hair_preview/assets.rs`: `436b82cac0058641b1c230f488b1ade01d293b23c6822584d39ed6846476e143`
- `crates/adventuresim-character-creator/src/hair_controls.rs`: `b412435396639226eabf4bb624b52eaa6c4ec25f1697b3b5987d569d7bf7440a`
- `crates/adventuresim-character-creator/src/character_export/hair.rs`: `ac11b7bb583e07eaa622342d8830d2bf203f8d3666706eb055af056f0abc3bd4`
- `crates/adventuresim-character-creator/src/main.rs`: `566a69fc6cd50bd7d7ea5c0347567c39f2bdc37e9f8004675dc5328e3751e12e`
- `crates/adventuresim-character-creator/src/hair_preview/render_test.rs`: `f141e4878ee8868ded50d8a481f8f1effb5c1ee4bb382fbcec321918c254c57e`
- `crates/adventuresim-character-creator/Cargo.toml`: `1346098aa561b83aa6d045c8744f19edc6605674fd0bbf313af4dcfc4ef595e5`
- `crates/adventuresim-character-creator/Cargo.lock`: `7e585078424d2838c9d07724cfa687802d6de81a4b1be33ca856b49122075a0c`
- `Cargo.lock`: `e76fa8ed410f3a4c8609d2b7528761832a8103ba075659ecfb3d1cc85a268d69`
- `crates/fabelgeist-mesh/Cargo.toml`: `b6a30a9372b2584916183463c9c88b45f69dc4f295c9de1c2428875036f73f94`
- `crates/fabelgeist-mesh/src/lib.rs`: `a1d2e436ae07dc79d25a7f0b670df3a3d7d3b3554d495db6542bd34f0c63f653`
- `crates/fabelgeist-mesh/src/count.rs`: `e609778b6bd24b4ee231f3d3281d463764a55d5b787f02f4015f4ff7c4b4072b`
- `crates/fabelgeist-mesh/src/count/gpu.rs`: `bcead2da272edc0fbcd1d285004d9671ad310603a5870bf1c7df114be337c4c4`
- `crates/fabelgeist-mesh/src/count/strip.rs`: `dd53d7fa66134977308f2dd0279c5d38bef8e907c739f6098904a40541a37208`
- `crates/fabelgeist-hair/src/cards/layout.rs`: `7affba536778dd1ae65da9b834c2400b2eddd4ce6bc6a7dc7aba5048a5fbf878`
- `crates/fabelgeist-hair/src/cards/build.rs`: `d0e1af5a572ecbdfdebcde495932dae897361eb446f4348824d6bcddd0cd2516`

## Material dependency fingerprints

- `crates/adventuresim-character-creator/src/hair/gpu/mod.rs`: `f8844fc7362364c5217079615e4459865865020ba4a36f5b25a8997cdd9c8f92`
- `crates/adventuresim-character-creator/src/hair_preview/gpu.rs`: `462eb12c93f953673bc177a3cef00835889022f59d278bf1884086fc328c2adc`
- `crates/adventuresim-character-creator/src/hair_preview/strand_shading.rs`: `13592232826ab0676fdf0cf93edd150713c436e3c73f743bd5d828072e1224b9`
- `crates/adventuresim-character-creator/src/hair_preview/hair_vertices.wgsl`: `62cbd9e3f606c85f4212494802e2675daa55387a27ac0133c8d79f80c0282b92`
- `crates/fabelgeist-hair/src/cards/layout/gpu.rs`: `6b7a180a3ee2faa7fc26a79e2ae14ceeac4a54c19f96a308dc57f417c56ea49f`
- `crates/fabelgeist-hair/src/cards/mesh.rs`: `322747ae7f6a10847b6f82813fd1b27cfd8caafb01f4d13aca401dc78346ab21`
- `crates/fabelgeist-hair/src/cards/atlas.rs`: `7cb59f0ac4dcfa2ea8643dc6384fdc15652d99d576f26a12fad2b43145cfc74b`
- `crates/fabelgeist-hair/src/cards/atlas/layout.rs`: `df403b648c0f3ba7f93f2f45c4deea2f33fc0c72fedef23d9f454397dbf428dd`
- `crates/fabelgeist-hair/src/gpu/lods/records.rs`: `7814ed1212fc58bea86aebf290862d5937f4e3e20d2b508050d20983948323a8`
- `crates/fabelgeist-hair/src/gpu/lods/cards.rs`: `77e815b2a949e96a4e90b90acc7f184021d395dcbb1881ee293fa0d94f2c69d6`
- `crates/fabelgeist-hair/src/gpu/layout.rs`: `ecdfc0cc3c30a0ec0b276f91a9799b7362430cd4ac9a281d208f68fcadc071a0`
- `crates/fabelgeist-hair/src/lod/policy.rs`: `16109d7ba1a21b99e1aaebad3835ed0dcd18d540dcd3e2961d9d05f706c857dd`
- `crates/fabelgeist-hair/src/lod/counts.rs`: `44ebb1a94560acc8633f707efbc477822c5c2f46fa4945b8ca640d41c7efc85e`
- `crates/fabelgeist-hair/src/parameters.rs`: `e8ddc101f1ef8295f3ab1943b4cae0257e483fd9fb67e2255bf614c1e51557e2`
- `crates/fabelgeist-gpu/src/data/gpu/buffer/prefix.rs`: `62a435f0e609deb067d2c9e59481ee7e042d0ee1f16c5d662f0ea6b4bd95b4c0`
- `crates/fabelgeist-gpu/src/data/gpu/buffer/read.rs`: `ff97b2b7c684745153c4713acc58e4d535cd97dd9008bcb8aacb9e62fdd138f2`
- `crates/fabelgeist-gpu/src/data/gpu/buffer/readback.rs`: `17ca9aa9db7efd1e261b086841babcf22764a6154628aca9405c085029b16baa`
- `crates/fabelgeist-gpu/src/data/gpu/buffer/stride.rs`: `9a8bfaa7b7b1d2d3c6c72d27cbd6f294c14356fee69bf72a59d4af38f89490b5`
- `crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs`: `86e926ab47cd284d07a0e14265aaf7b5ba3ce1b6cc5e2aff66328e05f67aee02`
- `crates/fabelgeist-math/src/units.rs`: `0c76a1a79a18e833d21fc2ffb011e2a351139ee7a299df13df12fdbcf3c27492`
- `crates/fabelgeist-compute/src/kernel/mod.rs`: `47b3faf12d1fdffc6251a9dcd840b28f8ce347bbf9e377e29671b4b171f9ff74`
