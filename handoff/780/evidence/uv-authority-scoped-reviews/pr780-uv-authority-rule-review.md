# Independent UV authority Rule + all-thirteen increment review

Verdict: **no new actionable Rule finding in the exact six-file increment**.
PBR-OUTPUT-2 is closed at source by the direct maps-only surface API, single
retained/remapped texcoords column and receiving admission below. This is not
a whole material/exporter/PR verdict and is not executed verification.

Reviewed at HEAD `59bb96d07af08244582c6767d81e969b820ec3ed` plus the six
working-tree files
below. No edits, checks, builds, tests, probes, new cases/helpers/assertions or
CI were performed. Source reads/comparisons and cryptographic bookkeeping only.
Applicable root/crates rules, Rule Review and thirteen-lens contracts govern.
The preinventory/peer reports were source maps, not inherited clearance.

## Exact source receipt

- `crates/adventuresim-character-creator/src/export/rigged.rs`:
  `f5fd53af89771488b500f8b2e661505b5b169dd355785fafe6005127f7c23fad`.
- `crates/adventuresim-character-creator/src/export/shell.rs`:
  `37d9aad2bed62e065620cccd542180cc439dba8aae9726d0b3177d9f7cb7311a`.
- `crates/adventuresim-character-creator/src/export/validation.rs`:
  `2f01964984a42e98201c8cb2be6095d0779a2f6292d8571021b64f70bc976656`.
- `crates/adventuresim-character-creator/src/character_export.rs`:
  `a542629ff77155448219d508e544a55f2da0c3f024f7454fb2c26a597f1da13a`.
- `crates/adventuresim-character-creator/src/equipment_export.rs`:
  `bc324c39eaed75e445e0b5247a8ce1adcd1cc336525d221ba6f174aacfb2cbd8`.
- `crates/adventuresim-character-creator/src/export.rs`:
  `e725590d8bfe60bfe165344b3476bacfc4ed84d7426fd7f59c2586a992fbf368`.

All six hashes match the ledger. Five changed files were read in full. In the
large export.rs, the actual export pipeline and complete changed textured
fixture were freshly read; source comparison proves its entire production
prefix unchanged and its entire existing test suffix unchanged except the two
API lines. Unrelated socket/numerical helpers and other fixture bodies are not
claimed as a fresh whole-file audit merely from their hash. Companion receipt:
`/tmp/pr780-uv-authority-rule-review-receipt.json`, including exact dependency
read extents and source comparisons.

## Source-backed contract and receiving trace

`export/rigged.rs` now has `surface: Option<&ShellTextures>` and one optional
per-vertex `texcoords` slice. Map identity has no vertex association to remap.
Its docs explicitly name the one column and the requirement for either map
payload. No old-name alias, tuple adapter or second UV path is retained.

`character_morphs::rigged_armor` supplies GeneratedArmor.texcoords for every
plate/trim shell at source construction. Character catalog export attaches only
matching map identity; trim explicitly replaces the selected texcoords with
its own physical-edge-produced chart, then attaches maps. Equipment export
similarly attaches maps without introducing another chart. Component material
overrides clear map payloads while retaining harmless UV data as before.
Draped mail/cloth and exported hair already carry their actual source UVs in
texcoords with generic map payloads. These unchanged producers were inspected
in their actual current receiving functions, not inferred from name searches.

`export_rigged_glb` calls validate before creating CompactShell, BufferBuilder,
image serialization or GLB output. `validation.rs:52-68` retains the plate-map
nonempty normal/metal-roughness requirement, then requires texcoords if either
map family is present. Every present UV column has exactly one finite row per
source position. Existing shell validation also admits face index membership,
geometry and skin correspondence before compaction. No normalization, fixed
UV range or new image decode policy is introduced.

`CompactShell::new` is byte-exact unchanged. Its used source-vertex BTreeSet
selects UV rows in precisely the same order as positions, normals, weights,
joints and morph deltas; face remapping uses the same membership/order. Its
reconstructed RiggedShell now carries only source map identity and compact-owned
texcoords. There is no source UV fallback left in surface. A source face using
rows [1,2,3] therefore receives selected UV rows [1,2,3], not the former
original
slice beginning at row zero. This is a static receiving-path proof, not a new
case or executed test.

`Writer::append` writes texcoords alone as little-endian f32 VEC2, with the
selected row count. Existing buffer alignment and component types are unchanged.
It then applies named PBR factors, generic SurfaceTextures, textile finish,
and finally plate ShellTextures in the same order. Generic color maps replace
base factor with white, alpha/sides/sampling remain their own policies, and
plate maps replace normal and metallic-roughness images plus set roughness
factor one as before. Image pointer cache identity is unchanged and maps need
no per-vertex remapping. Both independent map families remain permitted.

## Thirteen sequential increment dispositions

1. **Producer propagation**: CLEAR. Plate maps in character_export and
equipment_export are now maps-only references. Existing rigged_armor already
supplies the actual GeneratedArmor UV column; trim still replaces that same
column with its own selected trim UVs. Mail/hair generic maps already supply
their real texcoords. The existing textured fixture moves its identical uvs
slice to the canonical field. No forwarding/tuple adapter remains in this
repaired API.

2. **Existing type owners**: CLEAR. RiggedShell.texcoords owns the actual
per-vertex chart; ShellTextures owns reusable image bytes. CompactShell owns
selected UV rows with selected geometry. Anatomical SurfaceUvLayout is a
different socket mapping and is not reused. No new universal UV wrapper, mesh
schema or GPU quantity is introduced.

3. **Admission contracts**: CLEAR. Public source records are admitted in
export_rigged_glb before CompactShell or buffer/file output. Either generic
textures or plate surface maps requires texcoords. Every present UV column must
match positions.len() and contain only finite values. Existing plate map
normal/metal-roughness nonempty checks remain. Map-free UV-free shells remain
valid. UV values are not clamped or restricted to [0,1].

4. **Invariant operations**: CLEAR. The existing BTreeSet source-vertex order
selects UV rows exactly as positions/normals/skin/morphs; faces use that same
remap. Reconstruction retains selected texcoords and source maps only. Writer
serializes that selected column once as f32 VEC2 with its compacted row count.
No UV arithmetic or alternative remap is added.

5. **Aggregate states**: CLEAR. A surface map payload can no longer carry its
own stale, independently authoritative UV slice. The sole chart is correlated to
topology at public admission and through compaction. Maps have no vertex
population and remain borrowed safely until synchronous serialization. The
public map-without-chart state is still representable but rejected before native
use; no separate checked schema is claimed.

6. **Type ownership**: CLEAR. Offline export owns the per-vertex/chart
association; source geometry producers retain original charts, and TextureImages
owns images/cache identity. No concept moved into GPU/Compute/Math or physical
textile coordinates. Maps-only borrowing preserves actual image identity and
avoids duplicated chart ownership.

7. **Redundant authority**: CLEAR. surface.0 and Writer fallback are removed.
texcoords is the one current UV authority and CompactShell selects it once.
Existing TextureImages shell cache is legitimate image serialization state, not
a competing UV cache. The compactor source reference now retains maps/material
metadata while all per-vertex columns are compact-owned.

8. **Behavior preservation**: CLEAR for this increment. Existing explicit UV
choice remains canonical; former surface-only constructors must supply the exact
same slice in texcoords. The intentional correction is that a selected subset
now uses correspondingly selected UV rows. Native image bytes, cache identity,
material/alpha/sides/sampling order, geometry/morph/skin bits and source
ordering are unchanged. Error diagnostics may now come from the shared UV check
rather than the former surface-specific UV guard.

9. **Enum shapes**: CLEAR. Two map families are independent and may coexist;
making them exclusive would change existing behavior. Their two Options remain
valid. Optional UV availability belongs to public data admission, not a
material-role enum. Existing alpha/sides/sampling enums retain their meanings.

10. **Derivability**: CLEAR. No changed serde/default/manual trait
implementation exists in this borrowed shell API. Compact reconstruction is
semantic coordinated channel selection, not derivable copying. Existing image
Clone/cache Default behavior is unchanged. No new trait or serialization
boilerplate is needed.

11. **Type granularity**: CLEAR. Distinguish image identity from one chart,
rather than create one UV type/vector for every material family. Native float2
rows at this exporter boundary retain signed/outside-unit values. Host row
population, anatomical atlas identity and GPU launch/sort counts remain
separate.

12. **Magic strings**: CLEAR. TEXCOORD_0/VEC2 and JSON fields are concrete
native glTF keys; validation strings remain terminal diagnostics and no behavior
branches on them. No new string status, route or identity replaces a closed
role. Existing names/paths/fixture oracles remain unchanged.

13. **Magic numbers**: CLEAR. Pair/triangle arity, host source indices, native
f32/VEC2/component5126, ARRAY_BUFFER34962, little-endian packing and buffer
alignment are actual format/geometry boundaries. UV values receive no epsilon,
normalized bound or arbitrary capacity ceiling. Texture/alpha/material constants
retain their separate existing owners and are byte-exact unchanged.

## Rules, scope and existing fixture provenance

This is a direct final public Rust API change; RiggedShell is not a serde
record, so no extra wire decoder or compatibility path belongs here. Callers
that formerly supplied UVs only in surface must now use texcoords. That
intentional API redesign removes the old ambiguity; it does not silently
prefer a different stream when both old streams were supplied. Native GLB
UV output and texture protocols retain their current owners.

No new module, dependency, lock change, Result alias, broad suppression,
semantic primitive adapter, schema framework or ownership crossing is
introduced.
The six production deltas shrink/simplify relevant APIs; they do not materially
expand an oversized function/file or raise a baseline. Existing broad raw PBR,
semantic tuples, item-order and alias findings remain in the separately open
material groups; they are not newly cleared because these files were read.

Exact source comparisons in the receipt establish:

- CompactShell's entire source is byte-exact unchanged.
- TextureImages/texture policy source is byte-exact unchanged.
- Character export is otherwise byte-exact after only its two maps-only source
  substitutions; equipment export after its one maps-only substitution.
- export.rs production prefix is byte-exact unchanged. The entire existing
  fixture suffix is byte-exact after changing only the textured fixture's
  surface tuple to maps-only and moving its identical uvs slice to texcoords.
  This adaptation occurs once, with no helper, numeric input, case or assertion
  added or changed.

The existing textured fixture uses all four source vertices and retains all
its topology/skin/UV accessor/image expectations. It is not falsely described
as coverage of omitted-vertex remapping. No additional case is authorized.

## Explicit limits

Source-level PBR-OUTPUT-2 closure does not clear PBR-OUTPUT-1, the newly
source-backed finite encoded-color/native projection overflow, MCF-1 or the
broad SurfaceMaterial/ComponentAppearance/schema migration. The preserved
final API design report is unaffected. Density/Armory retains its own separate
completed increment review. Full socket math, physical algorithms,
map/rasterizer
numerics, every external Rust client and other unrelated exporter helpers were
not freshly audited as part of this narrow receiving repair.

No runtime failure-path test, compilation, formatting, lint, hosted CI or final
same-pushed-revision review pass is asserted. Root must compose this exact
source with the remaining connected migration families, independent reviews
and required hosted checks before final PR clearance. No merge is authorized.
