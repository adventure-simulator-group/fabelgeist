# UV authority tranche: independent Comment Review

Verdict: **CLEAR within this six-file, source-only scope.** No actionable
comment or documentation findings remain in the reviewed change. This is not
material-family or whole-PR clearance.

Snapshot: HEAD `59bb96d07af08244582c6767d81e969b820ec3ed` plus the six UV files below.
The unrelated dirty generated-program family is excluded. All six hashes match
`uv_authority_connected_preinventory.files` in the ignored ledger.

## Current contracts and evidence

- `export/rigged.rs:27-32` now correctly separates maps-only `surface` from
  the sole per-vertex `texcoords` column. Either optional map payload requires
  that column. `export/validation.rs:51-66` enforces presence, finite values and
  one UV row per source position before compaction. These docs describe the
  checked export boundary, not a constructor-enforced invariant.
- `export.rs:599-610` validates before constructing `CompactShell` values.
  `export/compact.rs:24-69,85-103` uses one sorted used-vertex set for positions,
  normals, UVs, skin and morph rows, and remaps faces with the same map. Its
  unchanged module documentation about identical channel remapping is now
  accurate for every emitted UV path: retained `surface` contains no UV array.
- `export/shell.rs:53-58` writes only the compacted `texcoords` column into
  `TEXCOORD_0`; there is no fallback to source rows. Its module comment remains
  accurate for independent shell topology, UVs, skin and textures.
- `character_morphs.rs:234-246` supplies armor UVs before the two direct export
  producers attach maps. `character_export.rs:162-203` retains the generated
  plate chart and explicitly supplies each trim's own along-edge chart.
  `equipment_export.rs:77-94` uses the armor chart. The trim comment still
  matches its actual source. Garment and hair constructors independently supply
  their own sole UV columns (`garment/export.rs:18`,
  `character_export/hair.rs:82`). Anatomical socket UVs are a separate layout
  contract and are not this per-vertex column.
- Texture precedence is preserved. `export/shell.rs:61-79` still applies scalar
  PBR factors, general surface maps, textile finish, then baked plate maps.
  The unchanged `export/textures.rs:173-235` shows plate maps overwrite the
  normal map and metallic/roughness map and set roughness factor to one;
  earlier base color, occlusion, alpha, sidedness and textile extension remain.
  No new documentation claims universal material admission or GPU completion.

## Source preservation

Each complete current changed file and its diff was read freshly. In-memory
reverse substitutions of only the declared API, validation and fixture moves
reproduce every complete saved before-file byte exactly. Each reverse site was
unique. In particular, reversing the existing textured garment fixture's move
from `surface: Some((&uvs, &textures))` to maps-only surface plus
`texcoords: Some(&uvs)` reproduces all of `export.rs` exactly. Its geometry,
texture values, assertions and helpers, all other fixtures, and all production
export code outside that fixture are unchanged.

The complete compactor, buffer and texture writer were freshly read and are
byte-exact to the snapshot. Whole-file byte comparisons also preserve assembly,
morph, texture and textile fixtures and the unchanged producer sources. The JSON
companion distinguishes fresh complete/partial reads from byte comparison only;
prior discovery verdicts were not used as current clearance.

## Final changed fingerprints

| Path | SHA-256 |
| --- | --- |
| `crates/adventuresim-character-creator/src/export/rigged.rs` | `f5fd53af89771488b500f8b2e661505b5b169dd355785fafe6005127f7c23fad` |
| `crates/adventuresim-character-creator/src/export/shell.rs` | `37d9aad2bed62e065620cccd542180cc439dba8aae9726d0b3177d9f7cb7311a` |
| `crates/adventuresim-character-creator/src/export/validation.rs` | `2f01964984a42e98201c8cb2be6095d0779a2f6292d8571021b64f70bc976656` |
| `crates/adventuresim-character-creator/src/character_export.rs` | `a542629ff77155448219d508e544a55f2da0c3f024f7454fb2c26a597f1da13a` |
| `crates/adventuresim-character-creator/src/equipment_export.rs` | `bc324c39eaed75e445e0b5247a8ce1adcd1cc336525d221ba6f174aacfb2cbd8` |
| `crates/adventuresim-character-creator/src/export.rs` | `e725590d8bfe60bfe165344b3476bacfc4ed84d7426fd7f59c2586a992fbf368` |

## Limits

No source edits, project checks, builds, tests, probes, new cases, helpers,
assertions or doctests were performed. Existing fixtures do not isolate both map
payloads simultaneously, or maps-only compaction with unused source vertices;
those paths were assessed from current source. All broader material, physical,
PBR ownership, native GPU and final combined PR reviews remain open.
