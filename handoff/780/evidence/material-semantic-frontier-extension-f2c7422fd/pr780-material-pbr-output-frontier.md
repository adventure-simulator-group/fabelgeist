# PR #780 independent PBR output frontier discovery

Read-only discovery before Root type edits. Snapshot HEAD: f2c7422fd6a0f995eb90b72cc9c2a5bd12de4d35.
No source edits, tests, builds, formatters, quality tools, execution probes,
new fixtures, or hosted checks. This is not implementation clearance.
The current Rule report was used as a source map, not a previous verdict.

## Concrete additional findings

PBR-OUTPUT-1: exported skin bypasses encoded-to-linear conversion.
Creator src/preview.rs:158 declares SKIN_COLOR as sRGB; skin_material uses
that color. src/export.rs:765-769 writes the same [.64,.39,.30,1] directly as
glTF baseColorFactor. Unlike the shell writer, it does not decode the sRGB
channels. Both receivers should consume canonical skin appearance and the
glTF projection should decode RGB once while preserving alpha. This is an
intentional output correction to inventory separately from mechanical type
propagation, not behavior to preserve accidentally.

PBR-OUTPUT-2: an accepted plate-only UV stream survives compaction unmapped.
export/validation.rs:52-58 admits surface=Some(vertex-sized finite UVs,
nonempty maps), texcoords=None. compact.rs:50-52 selects only shell.texcoords;
:88 copies source.surface with the original UV slice. shell.rs:53 then falls
back to those original UVs after geometry has been compacted. Four vertices
with face [1,2,3] become three vertices with four original UV entries starting
at vertex zero: wrong source association and unequal attribute lengths. This
is an accepted public export path; actual internal rigged_armor calls also
supply texcoords=Some, so no failing internal invocation is established.
Select the effective UV stream once with existing explicit-UV precedence,
compact it with all other per-vertex channels, and retain the maps without a
stale independently authoritative UV slice. No new test case was added or
requested by this report.

## Named aggregate and full production migration graph

Paths here are relative to crates/adventuresim-character-creator unless
another crate is named. The one shared aggregate must be the existing
ArmorComponentMaterial, refactored/promoted as agreed. Encoded float RGBA and
canonical ratios belong in it. No Creator-only competitor, tuple adapter,
compatibility alias, or retained independently mutable triple is needed.

1. fabelgeist-armor/src/components.rs:124 owns ArmorComponentMaterial;
   ArmorComponent.material retains Option of its Copy/PartialEq/serde value.
   Its 'Unlit' documentation is inaccurate: inspected consumers are lit PBR.
   Physical ShellMaterial and authored TextileAppearance are distinct owners.
2. clothing_material.rs:3 returns a tuple for all ten catalog variants;
   lib.rs publicly reexports it as equipment_pbr. Return the named aggregate
   directly. Keep each table value; polished steel consumes Metal::default.
3. clothing.rs:59-106 from_catalog -> for_surface then stores three raw fields
   in GarmentSpecification. Retain one appearance field instead. The shell
   constructor at :315 clones the entire specification, so this transfer must
   carry the aggregate without recomputing appearance/defaults.
4. Actual specification creators: outfit::clothing_specifications,
   EquipmentExporter::garment, runtime_equipment/clothing::generate and one
   existing clothing fixture. outfit feeds preview and character export.
   Runtime clothing drops the specification only when converting geometry to
   GeneratedArmor and subsequently derives appearance from catalog identity;
   it does not retain another primitive PBR triple.
5. character_morphs.rs:207-230 rigged_clothing copies specification appearance
   into RiggedShell. Copy the aggregate whole, with no conversion/default.
6. character_morphs.rs:233-279 rigged_armor creates plate and trim shells from
   repeated [.769,.776,.776,1], metallic one, roughness .20. These duplicate
   Metal::default. Consume a canonical appearance producer. Existing callers
   overwrite these placeholders, but the constructor must still be coherent.
7. character_export.rs:154-224 catalog_shells assigns catalog appearance,
   attaches underlayer/plate maps, applies component overrides, then emits
   trim and lacing shells from their own actual Metal/Lacing source values.
   Each triple mutation becomes one aggregate assignment. Trim metallic stays
   one; cord metallic stays zero; keep their own colors and roughness.
8. equipment_export.rs:77-96 performs the catalog plate/map/component sequence;
   its garment branch uses rigged_clothing. It intentionally exports catalog
   defaults without article trim. Migrate both export entry points together.
9. character_morphs.rs:185-203 component_materials first copies component
   appearance and clears textures, plate surface and plate edges. Its second
   independent branch then supplies textile finish and optional textile maps.
   Textile can reintroduce maps. Preserve order and the independent branches.
10. garment_material.rs:19,85-88,178: CLOTH_PBR, MailWeave::pbr and
    MailSurface::pbr are tuple producers/forwarders. Directly return/retain the
    named aggregate. Cloth is [.52,.42,.28,1], zero, .85. Mail is white, one,
    authored weave roughness because the color lives in its image.
    garment/export.rs:6-31 chooses this value and matching mail chart/maps.
    character_export::mail_surfaces is its actual caller; the public method's
    matching-garment/mail correlation remains a documented precondition.
11. character_export/hair.rs:76-95 directly constructs white nonmetallic hair,
    HAIR_ROUGHNESS, coverage Mask, Both sides, OpenGarment sampling. Preserve
    these independently of metal defaults. tinted_atlas :111 multiplies
    already-linear pigment by shade; :130 encodes RGB to image bytes; alpha
    stays coverage. Do not route the image through encoded PBR factor decoding
    a second time or quantize float factors through a u8 palette type.
12. export/rigged.rs:23-46 is the public retained receiver. Keep skin, morph,
    hinge and texture metadata intact while replacing the triple.
    export/compact.rs:85-104 is a separate production constructor and copies
    all appearance state; copy the new aggregate whole. Fix PBR-OUTPUT-2
    under its own preinventory.
13. export/shell.rs:12-80 is actual glTF projection: decode encoded RGB once,
    preserve alpha, project ratio scalars at JSON. export.rs:578's decoder is
    exactly fabelgeist-color::transfer::srgb_to_linear (same threshold,
    constants, operation order and powf), so remove competing ownership by
    consuming the canonical conversion through the agreed encoded color type.
14. export/assembly.rs and export/morphs.rs were fully read. Production reads
    names, hinges, boundaries or morph channels, not PBR appearance. Their
    direct RiggedShell construction is in existing fixtures only; adapt API
    literals without unrelated production edits or fixture logic changes.

## Inferred producers and receiving closure

Armor textile.rs:10 expands TextileColor's u8 encoded channels using
f32::from(channel)/255; alpha one, metallic zero, roughness .92. Preserve
precision and serialized palette representation. textile_pattern.rs:220 then
substitutes opacity, finish-dependent roughness and metallic-thread .8. Carry
the named aggregate and typed leaves throughout that operation.

Direct hardware constructors: cloth_fastener_mesh.rs:101,
cloth_hook_mesh.rs:128, cloth_lacing_mesh.rs:124. They take TextileColor's
encoded color and substitute metallic one plus authored roughness; ring
roughness is .35. ClosureMaterial.surface retains the shared aggregate;
FastenerMesh::component copies it into ArmorComponent.material. Button loops,
cord and sewn_eyelet_mesh use fabric.material and retain the actual fabric.
underlayer_armor fabric/trunk-hose partitions and waist_support_mesh component
construction also receive TextileColor/TextileAppearance values by inference.
Their material assignments must consume the canonical producer directly.

device_textile_materials::apply assigns fabric.material and fabric together;
its support branch can assign the support TextileColor instead. Gown material
selection is upstream fabric selection, not another primitive triple owner.
No geometrical/style correctness verdict is made for it.

Actual native raw-field receivers inspected:
- preview::spawn_clothing and spawn_armor component branch;
- preview::lacing_material;
- textile_surface::material (inferred fabric.material result);
- underlayer_preview::EquipmentMaps::material, including its metal path;
- metal_preview::MetalImages::material, where map owns roughness;
- drape_preview::MailMaterials::material and cloth_material;
- tactical-client equipment/runtime_equipment/meshes.rs component partition
  receiver and runtime_equipment_material catalog fallback;
- export::shell::Writer and direct body-skin writer.

Bevy uses Color::srgb/srgba for encoded factors. Tactical MaterialPartition
compares full appearance plus textile identity; retain PartialEq and Copy.
Textile takes precedence over component scalar appearance in tactical and
studio native preview. All producers and these inferred consumers must migrate
in one connected family; a search for type names alone would miss several.

## Admission, defaults, alpha, textures, and precedence

Generic PBR is not the Metal authored parameter policy. Catalog Wool and
TextileColor roughness .92 and existing fixture roughness one are accepted;
mail permits .05..=1. Do not apply Metal's .08..=.9 bounds to all materials.
Math Ratio is expressly unit-only and permits arbitrary scalar representation.
Wrapping the fields does not prove finite or renderable values. Existing
export rejects nonfinite appearance in validation.rs:154-156 but accepts
finite out-of-unit values. Any tightening requires explicit behavioral
inventory, rather than an implicit claim that Ratio enforces bounds.
ArmorComponentMaterial has no Default implementation now. Preserve its
Clone/Copy/Debug/PartialEq/serde contracts and absence/inheritance semantics:
component material None inherits catalog; it is not white/nonmetallic default.

TextureImages::apply at textures.rs:174-203 independently owns alpha, sides,
sampling and image effects. Some base-color image replaces factor with white;
None preserves the real factor (existing normal-only fixture). Normal/AO/maps
are linear numeric data; only encoded color maps use sRGB. AO is red channel
at strength one. Opaque/Mask cutoff .5/Blend are explicit policies independent
of scalar alpha; Front/Both and Tile/Wrapped/Open remain separate identities.

Writer order is named factors -> generic SurfaceTextures -> textile finish ->
plate ShellTextures. Textile finish adds KHR_materials_specular using actual
finish.specular_factor. Plate maps last replace normal and metallic-roughness
images, set roughnessFactor one, retain metallic/color factors. ShellTextures
pointer cache identifies actual maps, not equivalent scalar appearance.
Do not flatten these payloads into the simple PBR aggregate.

Textile maps bake encoded color and alpha, then native receivers set factor
white; without maps scalar opacity survives. Translucent cloth blends and
renders one outward wall. Mail color loads sRGB; its normal and AO load linear.
Hair retains coverage-mask policy and original coverage byte. Scalar/factor
migration must preserve these distinctions and publication/cache policies.

## Existing fixture API closure and acceptance

Direct existing RiggedShell literals occur in character_morphs (two production
constructors), garment/export, character_export/hair, export/compact, and
fixtures in export.rs (six), export/assembly.rs, export/morphs.rs,
export/textures/tests.rs and tests/textile_export.rs. Existing
export_equipment_with_maps accepts three primitive appearance arguments;
change that existing helper signature and callers to the aggregate as needed,
without adding a helper, case or assertion. Existing tactical mesh_tests.rs
has two aggregate literals; clothing.rs has one GarmentSpecification literal.
Fixtures were read for API closure; none were run or augmented.

Accept only after every producer, retained record, rebuilder and native
receiver above consumes the sole owner, with no tuple adapter/raw retained
triple. Preserve all ten catalog values, fabric opacity/finish, trim/cord
source overrides and texture precedence; fixtures retain existing inputs and
oracles aside from explicit source type API adaptations. Scalar sRGB decodes
exactly once at glTF; map formats keep their actual semantics. The two new
output findings require all-thirteen preinventory and explicit disposition
before source repair. Root's complete-family checks and independent reviews
on the same final pushed SHA remain later acceptance gates.

## Coverage limits and source identities

All seven specifically requested frontier files were fully read, including
existing fixtures. Direct production appearance paths described above were
read. Unrelated large numerical helpers in clothing.rs were not all reread;
only material-carrying sections/direct component construction of
underlayer_armor and waist_support_mesh were reviewed here. Export.rs entry,
appearance, construction and literal paths were inspected, not every socket
or binary-writing helper. Complete source hashes do not imply full-file
clearance for partial reads. No new verdict is made on geometry, simulation,
texture-rasterizer math, native scalp/Grid, generated programs, or broader
physical/material audit families. The companion JSON records the same hashes.

| Source | SHA256 |
|---|---|
| `crates/adventuresim-character-creator/src/garment/export.rs` | `0b57ba3d094cdb3ac0ea1e872f7177837ffae8c16be441a1dbcb823d220b26dc` |
| `crates/adventuresim-character-creator/src/character_export/hair.rs` | `bc1d25835f665c6c251087d1262e851bc01da86beda413c4d21b398b90cf594d` |
| `crates/adventuresim-character-creator/src/export/compact.rs` | `9b57fb4bbcebe5b20113a06d530507253693a37d40f85f1617a38cbccb63fa87` |
| `crates/adventuresim-character-creator/src/export/assembly.rs` | `69228ac01428944c7113f479834ac9179928ab25512d652a741cd799c562b494` |
| `crates/adventuresim-character-creator/src/export/morphs.rs` | `49339af440cea4592fc08969075def3ca3380fe9a495d0b77180049f879ceee3` |
| `crates/adventuresim-character-creator/src/character_morphs.rs` | `8f76594ec3871a68df2f8e98740aa629e479b1e6fc7a0ad9a63e4e7f029e30a1` |
| `crates/adventuresim-character-creator/src/export/rigged.rs` | `b53b6a874c7a23f38f7895c75a03c65c599c21e122d45b1843e5b3efa2b3a25e` |
| `crates/adventuresim-character-creator/src/export/shell.rs` | `93b0aa654eacc98d66c072b54cb1a1d0444b75e6bdc3cc0ecd7d76325f085bd6` |
| `crates/adventuresim-character-creator/src/export/validation.rs` | `61a7e389db804f2c17d7e619c27bb5bfc88f873f11e00d05097c85b6121518fb` |
| `crates/adventuresim-character-creator/src/export/textures.rs` | `1d5fcb4d3997b5c4137ad31b6fc871cc065f0b8ee3689cb87beb35e64b8e07fd` |
| `crates/adventuresim-character-creator/src/export.rs` | `82c083e495d9e3b992abf5ad23d3f99f48acc42da6900cd44ca99405140095de` |
| `crates/adventuresim-character-creator/src/clothing.rs` | `d3bce4f4d9a53afc7ea141c5edf1163e85f55cb1b455b976b3bd5e329902ce33` |
| `crates/adventuresim-character-creator/src/clothing_material.rs` | `879486f9beb55903e6b6788555437f70413b4449dbf5ebfcf6295b1a81e5d2b5` |
| `crates/adventuresim-character-creator/src/garment_material.rs` | `39b44062b3a3fb8db93c869721bcd0ede9f01f588cbde35ea6cdf4b9a26377e3` |
| `crates/adventuresim-character-creator/src/textile_surface.rs` | `6eb0a624f247ea8c1ed2ffd6b25f65ece4b8742d30dd44d190914c9e0b399726` |
| `crates/adventuresim-character-creator/src/preview.rs` | `f7fa8bef88e0b718991dab2245cce57436b6748186df1769f7fadcef64ceb698` |
| `crates/adventuresim-character-creator/src/character_export.rs` | `33f6b05450b7b1eb8e1080aa330b35696e6ee4c57da0bb4d208a18f66ebec963` |
| `crates/adventuresim-character-creator/src/equipment_export.rs` | `2906c6c5115f7924743595c1808b2337d4b167c27d301436ac22304c75479428` |
| `crates/adventuresim-character-creator/src/underlayer_preview.rs` | `48e7ef34b4f757817c54281efd85160d1920dcb9e61fd8536113fd0927e879ed` |
| `crates/adventuresim-character-creator/src/drape_preview.rs` | `c6e605a97a1a017db71939375f50395488a049834eebd77c4a09ee1058360077` |
| `crates/adventuresim-character-creator/src/metal_preview.rs` | `09af59779312ad82dd8a82bace7284c98503db7e55eab5018e1deee5dae3bf14` |
| `crates/adventuresim-character-creator/src/cloth_fastener_mesh.rs` | `bae2477655a7d54619d1f5265775f85fce6b4e1ec02693613f69a6b697ca3657` |
| `crates/adventuresim-character-creator/src/cloth_hook_mesh.rs` | `f0561bc0a88d651c54ed6c5d6334c2fdc9bb6b1e6d2408a35132eac95a07a131` |
| `crates/adventuresim-character-creator/src/cloth_lacing_mesh.rs` | `328fbd586aca214f25884be7bcf0d69be4ba4375e6b3baada8f4f6c4ddbb7b99` |
| `crates/adventuresim-character-creator/src/sewn_eyelet_mesh.rs` | `33ef4daa09c4bb138ca81a7cbcf163235670f05842c3671154ce5246d844fc95` |
| `crates/adventuresim-character-creator/src/device_textile_materials.rs` | `5d457ed74d03a8130d204d48803cb881c977c580d8e36a25ff94971dd2288f37` |
| `crates/adventuresim-character-creator/src/underlayer_armor.rs` | `0aeea0b47034016130f42d75d0cc2f050d6ab6447c71d05accff1c34e517cc32` |
| `crates/adventuresim-character-creator/src/waist_support_mesh.rs` | `3e272c03e30186833e70e0b054c64b4b153f852114a5a5ce03b4f1926e2c1b0f` |
| `crates/adventuresim-character-creator/src/outfit.rs` | `4d3281fb80e63eeb57bf3caaec77e63eb33d75c62e6ec5d7053e2fd76ebd95f2` |
| `crates/adventuresim-character-creator/src/runtime_equipment/clothing.rs` | `72974a2d1ab294dcd4b04a06a153d9781a559e9478b7f429ff32909fd6a03f8e` |
| `crates/adventuresim-character-creator/src/export/textures/tests.rs` | `43e385b5074a6c8884aabe7f27f70c016f9b16b16e19a0020d58cd01090de0b8` |
| `crates/adventuresim-character-creator/src/lib.rs` | `e468ea09b75d60d64e6b1d7612bc80f852b125d62c7473f34bfcd2a878f1b2dc` |
| `crates/adventuresim-character-creator/tests/textile_export.rs` | `3f0d6674ab64abd4e90c50836ccc7028edb39f616d8758b2c6ff25e8380d9059` |
| `crates/adventuresim-character-creator/Cargo.toml` | `1346098aa561b83aa6d045c8744f19edc6605674fd0bbf313af4dcfc4ef595e5` |
| `crates/fabelgeist-armor/src/components.rs` | `0cee8ea4265d2ec61509c2c7a516a7dfa6d9d46a483d83fdae928bd1fe010ad5` |
| `crates/fabelgeist-armor/src/textile.rs` | `15db7df94796b698c31fed85b836eddca6737ede411b7b1a4fcb88ed8f29dc30` |
| `crates/fabelgeist-armor/src/textile_pattern.rs` | `5eb511d902255a5f0b52b7f0de48232b1ee202e4e069cae7802be6cad5ec4f09` |
| `crates/fabelgeist-armor/src/material.rs` | `bf177dcb1b678b06f9174b3aa283e4aa7634638681ce88c8722a900052734d72` |
| `crates/fabelgeist-armor/src/lib.rs` | `d4b6a4af8b3137b359c2d08f8ef03445e100edc2d1280ca74ec816212af31264` |
| `crates/fabelgeist-armor/Cargo.toml` | `5a51ad1fc7ac2e848b3d8fd8b016f5e5d12c4e433c5ff600353d0530d0b6eba1` |
| `crates/adventuresim-tactical-client/src/equipment/runtime_equipment/meshes.rs` | `4994b1974149becac0c96f9e0dafeb0d26c1db776cc4adef9d2e1b388b033058` |
| `crates/adventuresim-tactical-client/src/equipment/runtime_equipment/mesh_tests.rs` | `142460bac2304c4a63a77c9759a003d9bd57a9d5b6210b8d7954105d6ee3e827` |
| `crates/fabelgeist-color/src/color.rs` | `ec95ce41e0de2b14f31c2d8a206461d2ed69f95183e18ae020c7ce956de86c74` |
| `crates/fabelgeist-color/src/transfer.rs` | `81b5c5f4a55d8191a178ea71fc618c968db91c2b601fa7b72e0cadf466ecad62` |
| `crates/fabelgeist-color/src/conversion.rs` | `1c16766feee62bbb80b38bdcc46c18cd7c732d6f2d796a4c26e228cb4eff6f1e` |
| `crates/fabelgeist-math/src/units.rs` | `0c76a1a79a18e833d21fc2ffb011e2a351139ee7a299df13df12fdbcf3c27492` |
