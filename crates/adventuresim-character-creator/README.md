# Character creator

Native, non-authoritative character design tool backed by `fabelgeist-mhr`. It
loads
Meta's Momentum Human Rig assets locally, exposes its 45 identity coefficients
and 72 expression coefficients, and previews the generated mesh in Bevy.

Install the pinned upstream assets into the ignored local authoring cache, then
start the creator from the repository root:

```powershell
just init-mhr-assets
just character-creator
```

The importer verifies Meta's MHR v1.0.1 release by size and SHA-256 and installs
the FBX rigs and model definition under `target/mhr-assets/v1.0.1/assets`.
That default cache is about 50 MB after extraction. Run
`just init-mhr-lod1-correctives` only when comparing the optional LOD 1
pose-corrective network; installing every corrective basis is an explicit
`scripts/init_mhr_assets.py --all-correctives` operation and consumes about
4 GB. Override the location with `--assets` or `MHR_ASSETS` when needed. The
downloaded archive and extracted source assets are not committed; deliberately
exported game and Cascadeur artifacts are tracked separately.

The default project is the canonical zero-coefficient MHR body. **Save recipe**
writes the current parameters to the selected recipe path (by default,
`assets_src/characters/mhr_base.json`). **Export rigged GLB** writes to
`assets_src/biped/unarmed/base.glb` by default. The export is a zero-animation
T-pose containing MHR's 127 joints plus the three Fabelgeist animation
attachments, both sets of skinning influences, and inverse bind matrices for the
saved body. Use `just export-mhr-base <staging-path>` to export the canonical
body without opening the studio, then prepare its runtime copy as described
below.

The zero-weight attachment joints follow MHR's side-prefix naming convention:
`l_weapon` is parented to `l_wrist`, `r_weapon` to `r_wrist`, and `c_camera`
to `c_head`. Each weapon joint is positioned halfway from its wrist toward the
corresponding `*_middle1` knuckle, placing it in the generated palm. The camera
joint is positioned at the midpoint of the generated eye joints. Their
rotations inherit the wrist or head without mirrored negative scale.

Use the left panel to edit, randomize, reset, save, load, and export. Drag the
viewport to orbit and use the mouse wheel to zoom. The tool defaults to MHR LOD
1 with pose correctives disabled, preserving facial and finger topology while
keeping edits interactive. The **Pose-corrective model** checkbox reloads the
selected LOD with or without MHR's corrective network for direct comparison.
Recipes contain model coordinates, not authoritative character state, and must
be regenerated and validated when connected to game creation.

The preview reads each LOD's authored `ByVertice/Direct` normals from its MHR
FBX. It stores those normals in local rest-surface frames and reconstructs the
frames from the final generated vertices, so authored shading follows identity,
expression, skinning, and optional pose-corrective displacement. Triangle-only
normal reconstruction is retained internally only to define those frames; it is
not sent to Bevy as the character's shading normal.

## Animation integration

The exported base establishes MHR's stable bone names and hierarchy as the
animation-pack contract. The preview keeps body identity separate from
animation. Prism's retargeting pipeline establishes the intended boundary:
import a clip into an engine skeleton, retarget model-space deltas through
semantic rig profiles, then encode the resulting MHR joint pose into MHR's 204
model parameters. Identity remains this recipe's 45 coefficients, so one
retargeted clip works for every generated body. The creator currently shows a
neutral pose; clip playback should reuse Prism's `Retargeter`, `MhrRig`, and
`MhrPoseEncoder`, including its T-pose reference and hinge correction, rather
than copying local rotations.

## Identity morphs in game

Character and procedural equipment exports contain 45 named identity morphs,
`mhr_identity_00` through `mhr_identity_44`, with position deltas in metres and
normal deltas. Zero weights reproduce the saved recipe; weight 1 adds one MHR
identity coefficient relative to that recipe. Expression coefficients remain
baked into its neutral face. Morphs deform the surface independently of skeletal
proportions. Neither form of appearance variation changes tactical physics or
combat stats.

Every primitive of a clothed character uses the same ordered channels. Fitted
clothing retains its base trim triangles when refitted for each sample.
Parametric armor uses the armor generator's corresponding body samples,
including its anatomical joint landmarks. Each recipe retains its authored
vertex and index correspondence across morph samples. Export fails if fitting
changes either.

Breastplate identity targets transfer body displacement through fixed
correspondence on the smooth carrier. Refined flute vertices interpolate that
coarse displacement, and corresponding inner/outer wall vertices share it.
This avoids accumulating independent nonlinear fitting corrections in signed
identity blends. It preserves carrier gauge vectors, not exact normal thickness
under arbitrary deformation; installed skeletal animation needs its own checks.


The tactical client derives bounded cosmetic weights from each persistent
character ID, consistently across clients and reconnects. Equipment uses its
current wearer's weights, updates when transferred, and returns to zero weights
when dropped. Mesh assets remain shared; weights belong to each instance.

## Skeletal proportions

Recipe version 4 also stores nine absolute MHR skeletal coefficients in
`proportions`, ordered as hip width, shoulder width, upper arm length, lower arm
length, upper leg length, lower leg length, spine length, neck length, and foot
length. The creator's **Skeletal proportions** controls use the pinned model's
limits. **Neutral** resets both surface and skeleton; **Randomize body** varies
both. Zero is MHR's reference skeleton.

Exports store `adventuresim_proportions` in joint-node extras: the recipe's
reference coefficients and local joint translation deltas in metres per unit
coefficient. These nine controls drive translations, including bone-length
offsets; hand scaling, asymmetric lengths, and pose correctives are not part of
this skeletal contract. Export validates the mapping against the MHR model.

In game, character IDs determine stable skeletal coefficients independently of
the surface morph seed. The pose buffer samples shared reference motion and
adds instance-owned joint offsets before terrain and limb IK. A neutral-foot
height correction raises or lowers the pelvis for different leg lengths. Neither
the shared animation cache nor inverse bind matrices are modified. Equipment
uses the wearer's joint entities, so skeletal deformation applies once through
skinning, in addition to its surface morphs; transfers follow the new skeleton
and dropped equipment returns to its exported shape.

Two additional equipment targets, `mhr_skeletal_spine_short` and
`mhr_skeletal_spine_long`, refit the shell at the spine-length limits. Their
position deltas subtract the movement already supplied by skinning, preventing
double deformation. Body primitives carry zero deltas for these channels. The
client interpolates from the exported reference to either endpoint. Cadence and
distance-to-phase curves are remeasured from each character's retargeted foot
trajectories when its proportions change.

Use the canonical zero-proportion body as the runtime animation reference.
Regenerate body and equipment from the same recipe. For the canonical game body:

```powershell
just export-mhr-base target/morph-assets/base.glb
just generate-procedural-equipment target/morph-assets/equipment
python scripts/prepare_rig_base.py target/morph-assets/base.glb assets/animations/biped/unarmed/base.glb
```

Inspect the staged equipment before copying its GLBs and manifest into
`assets/equipment/procedural`. The base preparation step preserves morph data.

## Parametric armor authoring

All armor catalog entries have authored parametric recipes. Preview, character
export and equipment export use the same recipe dispatch, fit and material.
Catalog loading rejects armor without a recipe. The geometry code lives in
`adventuresim-armor-model`; the creator owns MHR landmarks, smooth fit
envelopes,
and transfer of UVs, skinning and morph targets.

Use `--write-armor-designs target/armor-designs.json` to write the editable
helmet, limb and garment defaults. Pass `--armor-designs` with that file to
preview or export overrides. Keys are catalog IDs; a recipe must retain its
construction family and pass its parameter validation. The existing
`--breastplate-design` option controls the paired torso plates. Measurements use
millimetres and ratios use permille. The serialized design contributes to the
asset's design hash and generator version.

The breastplate editor provides Rounded, Central ridge, Peascod, and Fluted
starting points. Its `profile` controls projection, upper-chest recession, the
height of that projection, lateral fullness, medial ridge, and waist-point
drop/width. Waist projection is independent of chest fullness; the flange
follows the waist point without a discontinuity when chest projection height
changes. Opening, length, waist width, clearance, and flange controls remain
independent.

Set `fluting` to `null` for a plain plate, or provide the flute recipe. Count
(2–24), width (350–850 permille of pitch), depth (1–4 mm), spread, lower spread,
start/end heights, and end taper are independent controls. Width is a proportion
of spacing, not an absolute millimetre width: increasing count at fixed spread
makes the flutes closer and physically narrower. Changing width at fixed count
changes the flute/land ratio. Spread and width scale with the fitted wearer;
flute relief depth remains in millimetres. Both surfaces carry the relief;
plate gauge follows the smooth carrier's extrusion direction, rather than the
local flute normal. Unknown fields and invalid fade intervals are rejected.

[Example recipes and historical references](../adventuresim-armor-model/review/breastplate/README.md)
provide editable starting points. Each recipe has one front plate and one back
plate; a separate plackart or articulated waist plate requires a different
construction recipe. The upper armscye is a smooth boundary of the shell, and
the back returns seat over the front at the lower flanks.

For reproducible body-visible review, run the creator with
`--armor-review-dir target/armor-review/candidate`, then:

```powershell
blender --background --python scripts/render_armor_review.py -- target/armor-review/candidate target/armor-review/candidate/renders
python scripts/armor_review_boards.py target/armor-review/candidate/renders
blender --background --python scripts/check_armor_review.py -- target/armor-review/candidate
python scripts/assemble_armor_review.py target/armor-review/candidate
blender --background --python scripts/render_armor_review.py -- target/armor-review/candidate/assemblies target/armor-review/candidate/assemblies/renders
```

These exports include the actual body, triangles, runtime vertex normals and
design parameters. The four-view boards preserve those normals and use back-face
culling. The mesh checker reports closed-edge
winding, triangle area, material volume and sampled body distances; those local
distance signs are diagnostics, not a proof of continuous clearance. Assembled
views combine unchanged parts to expose interface problems. Static review does
not replace inspection of installed equipment under runtime animation.

Filtered equipment exports accept comma-separated IDs with `--equipment-item`
and require an empty staging directory. Run
`python scripts/check_parametric_armor_assets.py STAGING_DIRECTORY` to audit
actual GLB winding, skin weights, all 47 morph endpoints and representative
blends. Use `--allow-partial` only for a deliberately filtered export.

For static review of the exported identity morphs, run
`python scripts/export_armor_morph_review.py STAGING_DIRECTORY OUTPUT_DIRECTORY`.
This evaluates the actual body and armor GLBs at neutral, positive, negative and
mixed identity blends. It does not refit substitute geometry. Use the rendering
and mesh-check commands above on each resulting directory. Skeletal proportions
and animation still require the gameplay renderer.

The native `animation-viewer` supports `--armor-harness plate|mail|padded` with
`--hidden` for automated captures. It uses the shared gameplay equipment visual
plugin and waits for every installed GLB, material, skin and morph component;
unresolved assets fail the capture. Rebuild it after updating the equipment
manifest, then capture idle, walking and raised-guard scenarios.
