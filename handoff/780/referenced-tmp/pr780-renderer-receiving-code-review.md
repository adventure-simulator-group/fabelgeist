# Independent receiving Code Review: canonical renderer graph

Root base/HEAD: `e2162708fcf3885193db434300ed6c2999ab7607`.
Corrected incoming owner revision: `2b646df899ee73045795b129bfbade5dec7d4b84`.
Its three corrective file changes are reviewed against the prior
`162b152c47ea3824b4bbcdea01215fe5342f71f0` receipt; the other fifteen renderer
files are byte-identical to that receipt.
Reviewed the current uncommitted receipt against Root producer/error/feature
owners. All eighteen captured renderer files match the incoming ledger hashes.
Read only `renderer_162b_receiving_preinventory` and the specified source-proof
report from ignored output. No edits, project checks, builds, tests, hosted CI,
pushes or external posts performed.

## Findings after corrective receipt

No actionable receiving integration finding remains in the captured eighteen-file
graph. Rechecked both fixes against actual current Root owners:

- `render/atlas.rs` no longer imports HairGpuError or implements its AtlasError
  conversion. `gpu/error.rs::HairGpuError::Atlas` retains the sole canonical
  `#[from] crate::cards::AtlasError` conversion, including under GPU-only builds.
  Thus the renderer now consumes the typed existing owner without the conflicting
  implementation or a Backend-only substitute.
- `render/draws.rs` and `render/frame_uniforms.rs` now import Ratio/Metres from
  `fabelgeist_math::units`, where the actual owners are publicly defined. Root
  math exports remain unchanged; no compatibility alias was introduced.

Compared the exact owner successor diff: only these three import/conversion
changes occur. Rechecked the remaining renderer source/API/error/feature closure
against current Root producer methods, private module visibility, canonical
counts/units, existing simulation and head/card/atlas records. No further concrete
integration defect was demonstrated. This is static source inspection, not a
claim that a compiler or hosted check passed.

## Other receiving contracts inspected

- Host `set_groom` consumes sealed Arc levels and canonical card/atlas getters;
  full host layout admission precedes native prefix fitting. Native
  `set_gpu_groom` checks actual device association before publishing source,
  coverage, levels and card resources. Private producer invariants tie every
  level to the same allocating device, and native cards additionally check it.
- `levels.rs` admits logical prefix independently of positive native binding
  minima and checks storage limits, physical extent and usages before binding.
  Generic card triangle lists retain canonical vertex/corner quantities through
  `meshes.rs`; physical minima never become nonzero logical draw counts.
- `simulation.rs` checks actual first-source Arc identity, keeps guide population
  independent from the maximum resident drawn prefix, and uses a sealed
  SimFrame/binding for host installation. `simulation/frames.rs` rejects foreign
  host binding identity before writes and foreign native layout/allocation/body
  identity before pose/offset publication. Source ports match the current Root
  immutable level and frame APIs by inspection.
- Simulation buffer receipt checks logical and physical extents, storage limits
  and usages. Deformation/copy preflight preserves empty logical prefixes.
  Installing a replacement simulation reuses original undeformed rest captures;
  clearing copies logical bytes only. Publication follows normal direct queue
  submission and does not claim fallible-context validation or GPU completion.
- Deformation uses the canonical launch shape/device validation and per-level
  update states for selected main, shadow and blend levels. Frame progress is
  separate until normal submission. Readback uses current callback-progress
  owners and typed native image/error ports; feature wiring enables its existing
  optional image/futures dependencies under render.

No additional concrete integration defect was demonstrated during this scoped
review or corrective recheck. The supplied proof preserves all five native POD layouts, existing
renderer fixture bodies and complete math/three-WGSL sources; source receipt
fingerprint verification independently confirms exact incoming contents. This
does not claim tests passed or runtime/visual correctness.

## Coverage and limits

Read the complete new receiving modules and current producer interfaces involved
in levels/cards/head/simulation/error conversion, selected constructor/render/
frame publication bodies in mod.rs, existing fixtures and preserved shader/POD
contracts. Checked error-conversion closure and root unit exports directly,
rather than accepting incoming peer verdicts as integration clearance.

Creator's direct Bevy strip/policy receivers remain a known incomplete family,
excluded from new findings here. The earlier Viewer/Dataset renderer-setter
pending label was unsupported and is withdrawn: current caller inventory has no
actual external HairRenderer constructor/setter calls to migrate. Broader
physical/grid/lower families, final full audit waves, same-SHA combined
independent reviews and required hosted checks remain pending. This report is
not final combined clearance.

## Captured current renderer fingerprints

```json
[
  {
    "path": "crates/fabelgeist-hair/src/render/atlas.rs",
    "sha256": "ced0a6437d99514b8201506e998d66ea89aab6b49fb1d852f868abc859f0ec7f",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/deform.wgsl",
    "sha256": "2d5c548775571bc1218b878303e1fdece0a31976d9d97cab38e7da14b9847d6b",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/deformation.rs",
    "sha256": "d16ff4a30fd78f3333a1ffda6f48f58aad13dafc1019d0b39606d919ca311885",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/draws.rs",
    "sha256": "ae246a424240ef39d3a652178c14e8eb7ca5731ef13731eb91e8c896eaa7b162",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/frame_state.rs",
    "sha256": "939c92bd3d64e773ff1d5f5191379ae1b90447b870dea9f3329629e1bc532b3e",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/frame_uniforms.rs",
    "sha256": "a0dc615037daa6d47efae6b02044ec36b8e286ec9f6d78d07be3ece901f052e7",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/hair.wgsl",
    "sha256": "84455da0ef19a8af85e8e565aba48ffdaada9344db717c1ac9bdb70ad4c77330",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/head.rs",
    "sha256": "db18108641e9fd28874236f2a8827c62cc989877f32215a97c6dc9906abe08a1",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/levels.rs",
    "sha256": "b0894deeae06ed99b55e2aac522e806d241a7827062fc3d92b6e4346adf80fd0",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/math.rs",
    "sha256": "297582116bff991d2c1dc9db4fc6d079b5b3b301255c517108c35a1c9fa0f2c9",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/meshes.rs",
    "sha256": "411e1488a75b45c69cee1e6a8d270a21c8e7dc8779d7669a399e8f9cd3209469",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/mod.rs",
    "sha256": "052eb05f205e18ce867d925db0db38571dcc98c92f6d4230d715e326b347dfd4",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/post.wgsl",
    "sha256": "b172f372370ae20d7bc392540c1f1e526de6735d8d870af4d4698f63da888770",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/readback.rs",
    "sha256": "e729f9f80d6e21d046fec9219c281bd8958065a4dc773d66926ae4e92fecb25f",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/simulation.rs",
    "sha256": "85c56d293d73fd327e164b1656ac12ea82935b5e62cc0351a519a2cce0d4210c",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/simulation/buffers.rs",
    "sha256": "41dd184f1c5b749e3177cc664d3dab1a826917172b3460c916e6cd82b03f575c",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/simulation/error.rs",
    "sha256": "7fecda61235602d534724602ca18d9c1277cd0b7cafdc1e64c0c1848f021c529",
    "matches_corrected_incoming": true
  },
  {
    "path": "crates/fabelgeist-hair/src/render/simulation/frames.rs",
    "sha256": "a5fa573140cc5649732b7b3073deb1f8d229dd85e4422331ff36c8d0489b33a8",
    "matches_corrected_incoming": true
  }
]
```
