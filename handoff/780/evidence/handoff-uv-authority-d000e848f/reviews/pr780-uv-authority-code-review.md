# PR #780 scoped independent Code Review: sole UV authority

HEAD: 59bb96d07af08244582c6767d81e969b820ec3ed plus the six fingerprinted uncommitted source files.
Verdict: no actionable correctness defect found in this exact tranche.
PBR-OUTPUT-2 is addressed by the reviewed source representation and path.
This is not hosted-test validation, a complete later audit wave, whole-PBR
clearance, or final combined review on the same final pushed SHA.

No source edits, local project checks, tests, probes, cases, helpers or
assertions were added or run. The earlier density review remains preserved.
The before snapshot is target/pr780-audit/uv-authority-before-59bb96d07/source.

## Contracts and actual receiving path

Read the full current rigged/shell/validation/compact sources and exact six
file diffs; reread complete catalog-shell, standalone equipment armor,
component material and rigged-shell constructors, public export entry, the
existing textured-garment fixture, generic map, mail and hair receiving
responsibilities. The complete source family was read in the immediately
preceding sequential all-thirteen discovery; exact unchanged fingerprints
were verified here and used with current source reasoning, not its verdict.

RiggedShell.surface is now Option<&ShellTextures>; it cannot retain an
original vertex-sized UV slice. texcoords is the sole chart field, with
truthful public documentation about either map payload requiring it.
Existing imports and lifetimes fit the direct API: references to cached
CatalogMetals and the local standalone Option<ShellTextures> remain borrowed
through synchronous export; no ownership extension, clone or forwarding
adapter was introduced.

Both plate and generic maps require Some(texcoords). Every present column
must be finite and match positions.len(), before CompactShell or any buffer
output. The plate-map nonempty normal and metal-roughness byte checks remain.
Untextured UV-free shells and UV-bearing shells remain accepted. Finite
signed/outside-unit UV values are not clamped or rejected. Empty shell/skin/
face/morph admission policy is unchanged. No new PNG-content admission is
claimed by these UV checks.

The unchanged compactor builds a sorted used-source-vertex set, selects
positions, normals, UVs, skin and morph rows by that set, and remaps faces.
Its reconstructed shell copies maps only and borrows self.texcoords. The
Writer now emits only that selected column with native VEC2/f32/count data;
there is no tuple fallback capable of reintroducing unselected source rows.
Thus selected source vertices [1,2,3] receive exactly selected UV rows
[1,2,3], including the formerly broken maps-only public construction path
expressed in the final schema with its required sole texcoords column.

## Complete producer and payload closure

Character catalog plate assignment now receives only the found map object;
rigged_armor already supplied the exact GeneratedArmor.texcoords column.
Trim still explicitly replaces shell.texcoords with trim.texcoords and then
attaches its maps. Standalone equipment armor assigns metal.as_ref() directly
and also consumes rigged_armor's UV source. These remove both tuple producers
without introducing primitive projection/reconstruction or source ambiguity.

Component overrides still clear plate/generic maps before independent textile
map/finish application, while retaining the correct source chart. Plain
clothing has neither maps nor UV. Draped mail/garment and exported hair retain
their actual sole UV arrays with generic maps. No other production surface
assignment or direct constructor remains in the inspected source graph.

Writer still applies generic SurfaceTextures, then textile finish, then plate
maps. Combined map families remain supported on the same TEXCOORD_0. Image
bytes, alpha/sides/sampling, color-space handling, roughness/color factors,
map object pointer cache, accessor order and native byte alignment are
unchanged. Texture map identity remains separate from per-shell chart identity.
Anatomical SurfaceUvLayout for sockets remains its separate body-atlas owner.

## Exact source preservation

Whole export.rs is byte-identical to before after the sole existing fixture
adaptation: surface Some((&uvs,&textures)) -> Some(&textures), and texcoords
None -> Some(&uvs). The actual fixture input array, faces, skin, output checks,
images and every assertion/case/helper otherwise remain unchanged. It uses all
four source vertices, so its expected UV accessor row count remains four.
No new fixture was created to cover the corrected public counterexample.

All seventeen affected unchanged source accounts match discovery bytes,
including the whole compactor, TextureImages receiver, character_morphs
constructors, garment/hair constructors, native BufferBuilder, existing
assembly/morph/texture fixtures, texture producers, import and feature owners.
The JSON records each equality individually, not just an aggregate verdict.
No Cargo/lock/feature API change or protected generated-source change belongs
to this tranche.

## Limits and pinned files

No project checks ran. Existing hosted gates and independent final reviews
remain necessary later. PBR-OUTPUT-1, named material/color/ratio schema,
physical/grid/generated/native families and broader export geometry/input
admission remain open. Source identities below do not extend read coverage
beyond the explicit responsibilities in the companion JSON.

| Source | Review role | SHA256 |
|---|---|---|
| `crates/adventuresim-character-creator/src/export/rigged.rs` | changed source | `f5fd53af89771488b500f8b2e661505b5b169dd355785fafe6005127f7c23fad` |
| `crates/adventuresim-character-creator/src/export/compact.rs` | unchanged affected/dependency source | `9b57fb4bbcebe5b20113a06d530507253693a37d40f85f1617a38cbccb63fa87` |
| `crates/adventuresim-character-creator/src/export/shell.rs` | changed source | `37d9aad2bed62e065620cccd542180cc439dba8aae9726d0b3177d9f7cb7311a` |
| `crates/adventuresim-character-creator/src/export/validation.rs` | changed source | `2f01964984a42e98201c8cb2be6095d0779a2f6292d8571021b64f70bc976656` |
| `crates/adventuresim-character-creator/src/character_export.rs` | changed source | `a542629ff77155448219d508e544a55f2da0c3f024f7454fb2c26a597f1da13a` |
| `crates/adventuresim-character-creator/src/equipment_export.rs` | changed source | `bc324c39eaed75e445e0b5247a8ce1adcd1cc336525d221ba6f174aacfb2cbd8` |
| `crates/adventuresim-character-creator/src/export.rs` | changed source | `e725590d8bfe60bfe165344b3476bacfc4ed84d7426fd7f59c2586a992fbf368` |
| `crates/adventuresim-character-creator/src/character_morphs.rs` | unchanged affected/dependency source | `8f76594ec3871a68df2f8e98740aa629e479b1e6fc7a0ad9a63e4e7f029e30a1` |
| `crates/adventuresim-character-creator/src/garment/export.rs` | unchanged affected/dependency source | `0b57ba3d094cdb3ac0ea1e872f7177837ffae8c16be441a1dbcb823d220b26dc` |
| `crates/adventuresim-character-creator/src/character_export/hair.rs` | unchanged affected/dependency source | `bc1d25835f665c6c251087d1262e851bc01da86beda413c4d21b398b90cf594d` |
| `crates/adventuresim-character-creator/src/export/textures.rs` | unchanged affected/dependency source | `1d5fcb4d3997b5c4137ad31b6fc871cc065f0b8ee3689cb87beb35e64b8e07fd` |
| `crates/adventuresim-character-creator/src/export/assembly.rs` | unchanged affected/dependency source | `69228ac01428944c7113f479834ac9179928ab25512d652a741cd799c562b494` |
| `crates/adventuresim-character-creator/src/export/morphs.rs` | unchanged affected/dependency source | `49339af440cea4592fc08969075def3ca3380fe9a495d0b77180049f879ceee3` |
| `crates/adventuresim-character-creator/src/export/textures/tests.rs` | unchanged affected/dependency source | `43e385b5074a6c8884aabe7f27f70c016f9b16b16e19a0020d58cd01090de0b8` |
| `crates/adventuresim-character-creator/src/export/buffer.rs` | unchanged affected/dependency source | `c92b80be46c77268cf276c3686aaedbd70bba5be1e4f7d5eaef7041f865be104` |
| `crates/adventuresim-character-creator/tests/textile_export.rs` | unchanged affected/dependency source | `3f0d6674ab64abd4e90c50836ccc7028edb39f616d8758b2c6ff25e8380d9059` |
| `crates/adventuresim-character-creator/src/garment_material.rs` | unchanged affected/dependency source | `39b44062b3a3fb8db93c869721bcd0ede9f01f588cbde35ea6cdf4b9a26377e3` |
| `crates/adventuresim-character-creator/src/garment_material/rings.rs` | unchanged affected/dependency source | `6905eff437f36dde6d25965152bfde07ab25c57596f1d5068c14fb52fe75a222` |
| `crates/adventuresim-character-creator/src/textile_surface.rs` | unchanged affected/dependency source | `6eb0a624f247ea8c1ed2ffd6b25f65ece4b8742d30dd44d190914c9e0b399726` |
| `crates/adventuresim-character-creator/src/underlayer_material.rs` | unchanged affected/dependency source | `52ebd2ffe14baf52f9e8bdd18ecf23fb26b94b1cf52c655fbc58074cd18471b7` |
| `crates/adventuresim-character-creator/src/main.rs` | unchanged affected/dependency source | `566a69fc6cd50bd7d7ea5c0347567c39f2bdc37e9f8004675dc5328e3751e12e` |
| `crates/adventuresim-character-creator/src/lib.rs` | unchanged affected/dependency source | `e468ea09b75d60d64e6b1d7612bc80f852b125d62c7473f34bfcd2a878f1b2dc` |
| `crates/adventuresim-character-creator/Cargo.toml` | unchanged affected/dependency source | `1346098aa561b83aa6d045c8744f19edc6605674fd0bbf313af4dcfc4ef595e5` |
