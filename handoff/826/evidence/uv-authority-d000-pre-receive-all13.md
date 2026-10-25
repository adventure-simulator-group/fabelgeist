# UV authority receiving inventory

Pinned `d000e848f2f5bc28b0728c2cd664c727a41ecf50`, tree
`57cea419b31eac49981a90599b87e6990b407bcc`, compared with
`59bb96d07af08244582c6767d81e969b820ec3ed`. Root has not copied the packet. The
six complete bodies, all fixture blocks and full diffs were freshly read; exact
exported bytes match the manifest and immutable Git.

The JSON holds one shared exact source-account table: six candidate sources
(2714 lines), eleven freshly read complete unchanged sources and one explicitly
bounded MailSurface API receipt. Neither hashes nor the peer review notes
substitute for body coverage.

| Source | Lines | SHA-256 |
| --- | ---: | --- |
| `U1 crates/adventuresim-character-creator/src/export/rigged.rs` | 62 | `f5fd53af89771488b500f8b2e661505b5b169dd355785fafe6005127f7c23fad` |
| `U2 crates/adventuresim-character-creator/src/export/shell.rs` | 111 | `37d9aad2bed62e065620cccd542180cc439dba8aae9726d0b3177d9f7cb7311a` |
| `U3 crates/adventuresim-character-creator/src/export/validation.rs` | 162 | `2f01964984a42e98201c8cb2be6095d0779a2f6292d8571021b64f70bc976656` |
| `U4 crates/adventuresim-character-creator/src/character_export.rs` | 250 | `a542629ff77155448219d508e544a55f2da0c3f024f7454fb2c26a597f1da13a` |
| `U5 crates/adventuresim-character-creator/src/equipment_export.rs` | 344 | `bc324c39eaed75e445e0b5247a8ce1adcd1cc336525d221ba6f174aacfb2cbd8` |
| `U6 crates/adventuresim-character-creator/src/export.rs` | 1785 | `e725590d8bfe60bfe165344b3476bacfc4ed84d7426fd7f59c2586a992fbf368` |

UG1 is sole chart authority and maps-only propagation; UG2 is canonical
admission and actual GLB byte capacity; UG3 is complete coordinated
graph/rule/hosted acceptance. Per-group acceptance and dependency records are in
the JSON.

- **producer-propagation** (U1-U6; character_morphs, garment/export,
  character_export/hair): RiggedShell.surface now contains maps only. Plate
  producers retain rigged_armor chart, trim explicitly selects trim chart,
  garments use own/mail chart and hair uses card UVs. validate -> CompactShell
  -> Writer uses the same one vertex column. Sound bounded API increment;
  physical/material/creator caller graph remains required. Connected repairs:
  UG1,UG3.

- **existing-type-owners** (U1-U3; compact and textures): RiggedShell.texcoords
  is the sole vertex chart. ShellTextures owns image payloads; SurfaceTextures
  owns its distinct image/alpha/face/sampling policy. No new GPU byte/count
  owner is imposed on glTF numerical data. Existing wire and host quantity
  boundaries require UV-R2; broad material semantic owner work remains.
  Connected repairs: UG1,UG2.

- **admission-contracts** (U3:51-69,97-162; U6 export_rigged_glb): Either map
  family requires Some(texcoords); any provided column, mapped or unmapped, must
  match position rows and be finite. Public fields are admitted on every export
  before compaction/output. No other unchecked entry calls Writer from outside
  private export owner. Rejects malformed UVs correctly; stable local validation
  kinds remain UV-R1. Connected repairs: UG1,UG2.

- **invariant-operations** (compact.rs:24-104; U2; glb.rs:14-25): Compactor uses
  the same sorted retained source-index set for positions, normals, UVs, skin
  and morphs; it does not interpolate or rescale UVs. Writer serializes only
  that compacted UV population. Final native byte padding/total/wire conversion
  is unchecked existing UV-R2. UV remap operation coherent; native encoding
  capacity not clear. Connected repairs: UG1,UG2.

- **aggregate-states** (U1,U3; compact.rs; character_morphs): Removing surface
  UV payload prevents two disagreeing chart authorities. Private CompactShell
  retains the source maps with its remapped chart. Borrow lifetimes preserve
  source association; validation proves row membership before indexing. Both map
  payloads may coexist under the old material override order. No unrequested XOR
  policy or universal mesh/material certificate added. Connected repairs:
  UG1,UG3.

- **enum-shapes** (textures.rs; U1-U3): Existing GlbOutput and
  SurfaceAlpha/Sides/Sampling own closed packaging/material roles. Optional
  texcoords represents actual optional geometry channel; map-required relation
  is checked at export. bool export_body remains a retained policy field, not
  migrated by this increment. No new semantic boolean/role duplication; broader
  appearance/closed recipe work remains required. Connected repairs: UG1,UG3.

- **redundant-authority** (U1:27-32,U2:53-60; compact.rs:87-92): Second UV slice
  and fallback are removed without an adapter or legacy path. CompactShell
  retains a source reference for nonvertex metadata, but its returned shell
  cannot expose old unremapped surface UVs because surface is maps-only. Source
  counterexample of stale chart authority closed. Connected repairs: UG1.

- **behavior-preservation** (U4,U5,U6; textures.rs; four exact comparisons):
  Normal successful sources use the same chart and maps. Existing fixture moves
  the same uvs slice from surface into texcoords. SurfaceTextures then textile
  finish then ShellTextures application order remains exact; shell maps still
  override normal and metallic/roughness and roughnessFactor only. Missing
  texcoords with maps is intentionally rejected by clean final API; no fallback
  compatibility retained. Connected repairs: UG1,UG3.

- **derivability** (U1, textures.rs, compact.rs): No new
  wrapper/derive/default/serde admission route, Clone of native errors or unsafe
  trait assertion. ShellTextures remains borrowed maps; SurfaceTextures cloning
  preserves its existing owned payload semantics. No dependency/feature/lock
  mutation within six files. Error derives/owner for UG2 remain to settle.
  Connected repairs: UG1,UG3.

- **magic-strings** (U2,U3,U6; glb/textures/morphs): glTF JSON keys,
  type/semantic/sampler/format constants are actual wire protocol tokens.
  Names/paths are open identities or presentation. Stable input rejection
  messages are carried error classifications and remain UV-R1, rather than being
  exempted as arbitrary text. No new prose branching introduced; no claim all
  existing socket/export error roots are migrated. Connected repairs: UG2.

- **magic-numbers** (U1-U3,U6; buffer/glb/textures): f32 vector dimensions and
  authored numeric fixture/PBR inputs are distinct native/catalog roles. glTF
  accessor component/target constants and 4-byte alignment are actual protocol
  ports. Host-to-u32 length narrowing needs actual bound admission UV-R2.
  Existing material/tolerance/unit and semantic tuple debts are retained. No new
  UV scale, clamping, epsilon, count cap or precision policy. Connected repairs:
  UG2,UG3.

- **type-ownership** (U1-U6; actual producer and wire owner closure): Vertex
  chart remains with RiggedShell geometry; maps with textures; compaction owns
  remapping and Writer owns native accessor encoding. No duplicate
  type/helper/pass-through API. Export/character_export inherited file order and
  other modified-owner style work remains required. Selective composition must
  retain Root native and in-progress generated ownership. Connected repairs:
  UG1,UG3.

- **type-granularity** (U1-U3; native Writer/texture carriers): One per-vertex
  UV chart supports the current one TEXCOORD_0 protocol. Surface/material image
  policy is not collapsed into geometry or another universal checked object.
  Existing Views/accessor/CatalogMetals tuples and PBR primitive records remain
  broader named-record migration debt. This increment closes redundant chart
  authority without inventing material-specific UV wrappers. Connected repairs:
  UG1,UG2,UG3.

Required dependencies remain:

- Complete coordinated material22+6/PBR/color/component
  appearance/recipe/unit/count/seed/cause graph remains required; six maps-only
  files do not provide it.
- Receive matching Root-owned Creator composition and canonical
  async/native/physical/source/generated dependencies without overwriting Root
  native Buffer/context/MetalGpu or dirty generated work.
- UV-R1 export admission classifications, UV-R2 GLB byte bounds and existing
  modified-owner order/semantic records/production unwrap obligations require
  canonical-owner coordination before complete-family clearance.
- Global all13 and independent Code/Rule/Comment plus required hosted
  checks/baseline export must pass on the same final pushed SHA.

The maps-only increment is source-coherent. Whole-family correctness and
completeness are not clear: UV-R1/R2, broader semantic/style and canonical
integration obligations remain required. No source edit, project execution or
new fixture was performed.
