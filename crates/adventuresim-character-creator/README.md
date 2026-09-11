# Character creator

Native, non-authoritative character design tool backed by `fabelgeist-mhr`. It
loads Meta's Momentum Human Rig assets locally, exposes its 45 identity
coefficients and 72 expression coefficients, and previews the generated mesh in
Bevy.

Install the pinned upstream assets into the ignored local authoring cache, then
start the creator from the repository root:

```powershell
just init-mhr-assets
just character-creator
```

The importer verifies Meta's MHR v1.0.1 release by size and SHA-256 and installs
the FBX rigs and model definition under `target/mhr-assets/v1.0.1/assets`. That
default cache is about 50 MB after extraction. Run `just
init-mhr-lod1-correctives` only when comparing the optional LOD 1
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
`l_weapon` is parented to `l_wrist`, `r_weapon` to `r_wrist`, and `c_camera` to
`c_head`. Each weapon joint is positioned halfway from its wrist toward the
corresponding `*_middle1` knuckle, placing it in the generated palm. The camera
joint is positioned at the midpoint of the generated eye joints. Their rotations
inherit the wrist or head without mirrored negative scale.

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
coarse displacement, and corresponding inner/outer wall vertices share it. This
avoids accumulating independent nonlinear fitting corrections in signed identity
blends. It preserves carrier gauge vectors, not exact normal thickness under
arbitrary deformation; installed skeletal animation needs its own checks.


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
the surface morph seed. The pose buffer samples shared reference motion and adds
instance-owned joint offsets before terrain and limb IK. A neutral-foot height
correction raises or lowers the pelvis for different leg lengths. Neither the
shared animation cache nor inverse bind matrices are modified. Equipment uses
the wearer's joint entities, so skeletal deformation applies once through
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

Inspect the staged equipment before copying its GLBs, shared PNGs and manifest into
`assets/equipment/procedural`. The base preparation step preserves morph data.

## Parametric armor authoring

All armor catalog entries have authored parametric recipes. Preview, character
export and equipment export use the same recipe dispatch, fit and material.
Catalog loading rejects armor without a recipe. The geometry code lives in
`adventuresim-armor-model`; the creator owns MHR landmarks, smooth fit
envelopes, and transfer of UVs, skinning and morph targets.

The authored helmet, limb and garment defaults live in
[`assets_src/equipment/armor-designs.json`](../../assets_src/equipment/armor-designs.json).
The paired torso and vambrace defaults live beside it in
`breastplate-design.json` and `vambrace-design.json`. The creator embeds these
authored inputs and validates every recipe before use. Edit the catalog, rebuild
the creator, then regenerate equipment assets to change the game's default
shapes. Invalid entries fail explicitly; there is no generated default fallback.

Use `--write-armor-designs target/armor-designs.json` to write the editable
helmet, limb and garment defaults. Pass `--armor-designs` with that file to
preview or export overrides. Keys are catalog IDs; a recipe must retain its
construction family and pass its parameter validation. Use `--bracer-design` for
the vambrace and `--breastplate-design` for the paired torso plates; these are
separate recipe files, outside the catalog override map. The editor's **Save all
armor designs** button writes the catalog, vambrace and breastplate recipes to
the three displayed paths. Each path must be distinct and its parent directory
must exist. Pass all three files back through their corresponding options to
reproduce the saved set in preview or export.

Measurements use millimetres and ratios use permille. The sallet's
`opening_width` is an angular exception: it is the face-opening half-angle in
milliradians. The serialized design contributes to the asset's design hash and
generator version. Generate current defaults before editing; recipe files must
include the required fields of the current schema.

Metal recipes expose construction-specific shape controls. Helmet crowns have
fullness, ridge height and optional fluting; sallets add face-opening width and
sweep, tail shape, and separate visor side-panel depth. Limb plates expose
section shape and edge flare, greaves add ankle extension, and joint cops add
wing shape and notch depth. Tassets have width, separation, inner cutaway and
rounded or pointed hems; gorgets have collar height, independent front/rear
depth and width, independent front/rear hem flatness, rear sweep and separate
neck clearance. Gorget fluting follows the front bib and leaves the shoulder
return plain. These controls shape fitted carrier surfaces, with physical
padding clearance and metal gauge kept separate.

The shared `PlateFluting` recipe applies to metal limb and garment plates,
vambraces, helmet crowns and close-helmet visors. Set the appropriate `fluting`
field to `null` for a plain surface (`crown.fluting` or `visor_fluting` on
helmets). Its fields are `count` (2–24), `width` (350–850 permille of pitch),
`depth` (1–4 mm), `spread` (400–850), `lower_spread` (500–1000), `start`, `end`,
and `fade` (100–250). The pattern runs from lower to upper plate coordinates;
start/end must remain within 50–950 and leave room for both fades. Lower spread
controls the fan at the bottom of the pattern. Mail and textile recipes do not
accept metal fluting.

The breastplate editor provides Rounded, Central ridge, Peascod, and Fluted
starting points. Its `profile` controls projection, upper-chest recession, the
height of that projection, lateral fullness, medial ridge, and waist-point
drop/width. Waist projection is independent of chest fullness; the flange
follows the waist point without a discontinuity when chest projection height
changes. Opening, length, waist width, clearance, and flange controls remain
independent.

The breastplate uses the shared flute recipe with a torso-specific distribution.
Set `fluting` to `null` for a plain plate, or provide that recipe. Count (2–24),
width (350–850 permille of pitch), depth (1–4 mm), spread, lower spread,
start/end heights, and end taper are independent controls. Width is a proportion
of spacing, not an absolute millimetre width: increasing count at fixed spread
makes the flutes closer and physically narrower. Changing width at fixed count
changes the flute/land ratio. Spread and width scale with the fitted wearer;
flute relief depth remains in millimetres. Both surfaces carry the relief; plate
gauge follows the smooth carrier's extrusion direction, rather than the local
flute normal. Unknown fields and invalid fade intervals are rejected.

[Example recipes and historical references](../adventuresim-armor-model/review/breastplate/README.md)
provide editable starting points. Each recipe has one front plate and one back
plate; a separate plackart or articulated waist plate requires a different
construction recipe. The upper armscye is a smooth boundary of the shell, and
the back returns seat over the front at the lower flanks.

For reproducible body-visible review, run the creator with `--armor-review-dir
target/armor-review/candidate`, then:

```powershell
blender --background --python scripts/render_armor_review.py -- target/armor-review/candidate target/armor-review/candidate/renders
python scripts/armor_review_boards.py target/armor-review/candidate/renders
blender --background --python scripts/check_armor_review.py -- target/armor-review/candidate
python scripts/assemble_armor_review.py target/armor-review/candidate
blender --background --python scripts/render_armor_review.py -- target/armor-review/candidate/assemblies target/armor-review/candidate/assemblies/renders
```

These exports include the actual body, triangles, runtime vertex normals and
design parameters. The four-view boards preserve those normals and use back-face
culling. The mesh checker reports closed-edge winding, triangle area, material
volume and sampled body distances; those local distance signs are diagnostics,
not a proof of continuous clearance. Assembled views combine unchanged parts to
expose interface problems. Static review does not replace inspection of
installed equipment under runtime animation.

Filtered equipment exports accept comma-separated IDs with `--equipment-item`
and require an empty staging directory. Run `python
scripts/check_parametric_armor_assets.py STAGING_DIRECTORY` to audit actual GLB
winding, skin weights, all 47 morph endpoints and representative blends. Use
`--allow-partial` only for a deliberately filtered export.

Individual equipment GLBs reference shared `texture-<BLAKE3>.png` files beside
them. Install those PNGs from the staging directory together with the GLBs;
their content-addressed names let the runtime reuse each image across pieces.
Base color remains sRGB, while normal and ambient-occlusion maps remain linear
through their glTF material roles. Assembled character exports embed their maps
and remain standalone files.

For static review of the exported identity morphs, run
`python scripts/export_armor_morph_review.py STAGING_DIRECTORY OUTPUT_DIRECTORY`.
This evaluates the actual body and armor GLBs at neutral, positive, negative and
mixed identity blends. It does not refit substitute geometry. Use the rendering
and mesh-check commands above on each resulting directory. Skeletal proportions
and animation still require the gameplay renderer.

The native `animation-viewer` supports `--armor-harness` with `plate`,
`plate-tassets`, `plate-underlayers`, `underlayers`, `mail`, `padded` and
`close-helmet`, plus `--hidden` for
automated captures. `plate-tassets` replaces the fauld with tassets because
those defenses share a rigid-armor catalog slot. It uses the shared gameplay
equipment visual plugin and waits for every installed GLB, material, skin and
morph component; unresolved assets fail the capture. Rebuild it after updating
the equipment manifest, then capture idle, walking and raised-guard scenarios.

## Equipment material UVs

`just generate-procedural-equipment DIRECTORY` includes an offline Blender
unwrap and normal/AO bake after geometry export. Set `BLENDER_BIN` to the
Blender executable when it is not on PATH. `just unwrap-equipment DIRECTORY`
applies the unwrap step to
an existing export; run it again after changing geometry parameters. Direct
creator CLI exports contain construction UVs until this finishing step runs.

The material atlas occupies `TEXCOORD_0`; existing anatomical coordinates move
losslessly to `TEXCOORD_1`, with their domain and channel recorded explicitly.
Socket coordinates are independent and unchanged. UV0 also gives runtime blood
decals a nonoverlapping surface on each plate. Sharp rims and transitions
between front, side, and rim-facing
zones separate plate faces from edge walls. Concealed rear
meridians and arm undersides provide cuts through curved panels. Blender's
angle-based solver unwraps those charts and packs them with a 0.004 UV margin.
Each independently articulated component has its own atlas. Layouts need not
remain identical between parameter configurations. Body-conforming textured
mail and padding keep their existing material coordinates.

Seam splits copy all original vertex and morph attributes without changing
triangle order, the rig, or component hinges. Tangents use the material UV
channel. Material textures select glTF `texCoord: 0` for these atlases.
Compare source and finished exports with
`python scripts/check_armor_uvs.py ORIGINAL_DIRECTORY FINISHED_DIRECTORY` to
check correspondence, nondegenerate charts, overlap, and tangent frames.

`just bake-equipment DIRECTORY` bakes an already unwrapped export. Normal maps
encode the detailed mesh normals, including fluting, relative to a smoothed
shading carrier. The export carries those low-frequency vertex normals and the
matching tangent frame. Each morph endpoint receives the same smoothing rule;
positions and skin weights are unchanged. AO comes from Cycles rays against the
actual plates within the item. Maps are separate linear glTF normal and
occlusion channels, both using UV0; the unlit albedo is unchanged.

The default bake uses 1024-square images, 32 AO samples, and two-pixel gutters.
`scripts/bake_armor.py` exposes resolution and sample controls for offline work.
Unused AO atlas space is white to avoid dark mip bleeding.
Texture filenames are content-addressed. Regenerate geometry before changing
an already baked atlas; repeatedly smoothing an existing bake is unsupported.
Compare source and finished exports using `scripts/check_armor_bakes.py`.

A simplified mesh must preserve the carrier normals and material coordinates,
and regenerate a matching tangent frame; recomputing geometric normals would
apply the flute relief twice. `scripts/render_armor_materials.py` demonstrates
this with a static decimation and UV-based carrier-normal transfer.
The reduction ratio is adjustable; validate thin plate walls after
simplification. It does not implement runtime LOD selection. Cycles previews
ray trace ambient occlusion rather than
multiplying the exported AO map into albedo; runtime glTF uses the separate AO
channel for ambient lighting.

## Body-conforming underlayers

`arming_doublet`, `padded_chausses`, `mail_voiders`, `mail_brayette`,
`mail_knee_voider`, and `mail_standard` use the `Underlayer` recipe. The doublet
and hose occupy the padding layer. Voiders attach to the
doublet's `mail_voiders` attachment point in the flexible-armor layer. Removing
plate reveals the same clothing and mail; exposed cloth never changes material
automatically. The doublet now covers the torso and sleeves, so separate quilted
sleeves conflict with it. Captive plate or helmet linings are not independent
equipment articles. The brayette independently covers the groin, seat, and
upper thighs. Knee voiders attach to the matching left or right padded hose;
removing that hose also removes its attached mail. The optional mail standard
protects the neck. The plate fixture uses its gorget instead of the standard.

The mesh follows source body triangles with outward standoff. Shading normals
are corrected against incident face planes before offsetting. Include volumes
and Boolean subtraction boxes split crossed triangles, interpolate their source
UVs and skin weights, and close the cut edge with an inner surface. Uncut source
triangles retain their connectivity. A frozen barycentric cut plan gives every
morph the same vertices, UVs and triangle indices. Canonical UV seams remain
split for texturing and physically joined at the garment surface.

Offset directions also satisfy the sampled unposed proportion shapes. The
shared layer envelope accounts for both local folds and nearby opposing body
surfaces, including the space between the thighs. Bone-translation checks use
the same reference offset vectors that the exported mesh retains. These fitting
constraints do not repair a self-intersecting source body.

Recipe distances are millimetres. `clearance` accepts 1–10 and `thickness` 1–8.
They describe the uncompressed stack: tight concave body creases reduce both
distances together using a shared body field. Doublet `length` accepts 700–1200
permille; hose length accepts 700–1000. `sleeve_length` accepts 500–1000 and
`patch_width` 35–100 mm. `cuts` contains boxes with `minimum` and `maximum`
three-coordinate points in the reference body's metre space. Cuts follow the
body after morphing rather than remaining stationary in world space.

Mail uses shared color/alpha, tangent normal, and ambient-occlusion atlases in
the body's canonical UVs. Albedo contains a uniform unlit steel color;
ring relief and recess shading belong to the normal and AO maps. Normal and AO
channels are linear data; color is sRGB. AO is bound to the native material
and glTF occlusion channel, never multiplied into albedo. A construction unwrap
carries the weave around curved panels;
the atlas is baked back into the original UV layout. Separate torus rings
provide the weave bake, including overlap and apertures. Production meshes
remain lightweight skinned surfaces, not individual animated links. Regenerate
a carrier covering every mail family at maximum width and length before
rebuilding an atlas:

```powershell
blender --background --python scripts/unwrap_underlayer_charts.py -- MAIL_REVIEW_DIRECTORY target/mail/charts.json
blender --background --python scripts/bake_mail_weave.py -- target/mail/weave
python scripts/bake_underlayer_materials.py target/mail/charts.json target/mail/weave assets_src/equipment/materials
blender --background --python-exit-code 1 --python scripts/check_underlayer_assets.py -- STAGING_DIRECTORY assets/animations/biped/unarmed/base.glb target/mail/intersections.json
blender --background --python-exit-code 1 --python scripts/check_underlayer_plate_interfaces.py -- STAGING_DIRECTORY assets/equipment/procedural target/mail/plate-interfaces.json
```

The intersection checker tests exported triangles against the body and adjacent
underlayers, all identity basis targets, signed runtime identity bounds,
representative blends, and skeletal proportion endpoints. It applies skeletal
fit residuals together with bone translations. It supplements the common GLB
topology audit and native animation fixture; sampled tests do not establish a
continuous guarantee over all possible bodies and motions.

Repeat `--only CONFIGURATION_NAME` to run a focused set of unposed cases. The
report records source-body self-intersections separately; source defects never
exempt a garment from its intersection checks.

Use `--pose-trace PATH/global-bone-transforms.jsonl` to audit eight sampled
frames from an actual animation-viewer capture. Keep its `armor-readiness.json`
and `body-proportions.json` beside the trace: they supply the captured wearer
configuration. Both proportion and pose audits use the four normalized skin
influences consumed by Bevy. Exports merge joint
influences, retain the four strongest, and renormalize them before writing
`JOINTS_0` and `WEIGHTS_0`. The runtime consumes those four coefficients
directly; the checker does not divide by a homogeneous weight sum. Linear skinning
can
produce intersecting folds, including folds already present in the source body;
the underlayers do not provide cloth collision resolution. Keep a failing
intersection report distinct from a successful asset-loading capture.

The neutral plate fixture reserves space for these defaults through its cuirass,
gorget, fauld, and spaulder clearance parameters. Cuirass section fitting
preserves the requested front and back clearance after seating its returns.
Increasing garment thickness still requires checking the assembled kit; changing
an underlayer does not automatically refit every equipped plate.

Construction references are the Philadelphia Museum of Art's
[arming doublet, 1977-167-240, c. 1550–1650](https://www.philamuseum.org/objects/71390),
and the Met's German sixteenth-century
[armpit defense, 27.183.35](https://www.metmuseum.org/art/collection/search/34856).
The latter supplies the mail link dimensions: 6.2 mm outside and 4.0 mm inside
diameter. The Met's
[half sleeve, 29.158.186](https://www.metmuseum.org/art/collection/search/23246)
describes attached armpit and elbow gussets. These support separate textile
foundations and localized mail, not a universal requirement for a full mail
shirt beneath plate. The fitted hose is a simplified game garment; it is not a
reconstruction of a photographed surviving padded hose. Period arming dress
varied, including ordinary hose with local knee padding. Closures, stitching,
and individual lacing cords are simplified in these meshes.

The brayette's broad hip and seat wrap with a perineal bridge follows the
[Met's German sixteenth-century brayette, 27.183.14](https://www.metmuseum.org/art/collection/search/34839).
The optional standing collar follows
[Met 27.183.8](https://www.metmuseum.org/art/collection/search/34834).
Longitudinal knee strips adapt the surviving mail in Bavarian Army Museum
A 6147, illustrated in
[Christopher Retsch's catalogue, pp. 190-211](https://d-nb.info/139208119X/34).
That surviving hose contains sewn-in plates; it is evidence for the mail strip
arrangement, not a claim that the game's padded hose replicates that garment.
