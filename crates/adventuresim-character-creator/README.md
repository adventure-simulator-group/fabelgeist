# Character creator

Native, non-authoritative character design tool backed by `fabelgeist-mhr`. It loads
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
viewport to orbit and use the mouse wheel to zoom. The
tool defaults to MHR LOD 1 with pose correctives disabled, preserving facial
and finger topology while keeping edits interactive. The **Pose-corrective
model** checkbox reloads the selected LOD with or without MHR's corrective
network for direct comparison. Recipes contain model coordinates, not authoritative character
state, and must be regenerated and validated when connected to game creation.

The preview reads each LOD's authored `ByVertice/Direct` normals from its MHR
FBX. It stores those normals in local rest-surface frames and reconstructs the
frames from the final generated vertices, so authored shading follows identity,
expression, skinning, and optional pose-corrective displacement. Triangle-only
normal reconstruction is retained internally only to define those frames; it is
not sent to Bevy as the character's shading normal.

## Animation integration

The exported base establishes MHR's stable bone names and hierarchy as the
animation-pack contract. The preview keeps body identity separate from animation. Prism's retargeting
pipeline establishes the intended boundary: import a clip into an engine
skeleton, retarget model-space deltas through semantic rig profiles, then
encode the resulting MHR joint pose into MHR's 204 model parameters. Identity
remains this recipe's 45 coefficients, so one retargeted clip works for every
generated body. The creator currently shows a neutral pose; clip playback
should reuse Prism's `Retargeter`, `MhrRig`, and `MhrPoseEncoder`, including its
T-pose reference and hinge correction, rather than copying local rotations.

## Identity morphs in game

Character and procedural equipment exports contain 45 named identity morphs,
`mhr_identity_00` through `mhr_identity_44`, with position deltas in metres and
normal deltas. Zero weights reproduce the saved recipe; weight 1 adds one MHR
identity coefficient relative to that recipe. Expression coefficients remain
baked into its neutral face. Morphs deform the surface independently of skeletal
proportions. Neither form of appearance variation changes tactical physics or
combat stats.

Every primitive of a clothed character uses the same ordered channels. Fitted
clothing retains its base trim triangles when refitted for each sample. Parametric
vambraces and breastplates use the armor generator's corresponding body samples,
including their anatomical joint landmarks.

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
