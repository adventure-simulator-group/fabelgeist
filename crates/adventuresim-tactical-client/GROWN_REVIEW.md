# Review shared grown tree meshes

The native tactical viewer places three grounded instances of one admitted
source tree. It installs the selected mesh and material closure once, then
shares the handles between placements. The diagnostic checks actual indirect GPU
draws and compacted instance records before saving an image. Each process holds
one explicit Full, Middle or Coarse level throughout the run. Full uses the
library's declared detail; Middle uses six tube sides, one span and six leaf
blades; Coarse uses four sides, one span and two blades. The same admitted
records and botanical source identity are used at every level.

This does not replace the gameplay tree presenter, select adaptive levels of
detail, or demonstrate arrival performance.

## Input and commands

Supply a checked `library.forest`, its actual botanical `PlantId`, and a
tactical scene input. The identifier is neither a model ordinal nor a placement
ID. The selected model must contain both branch and leaf records. Keep source
generation evidence separately: decoding cannot establish how records were
produced, biological age, or species calibration.

The library must use `FORESTL3`, with its exact canonical topology revision.
It contains source records and model metadata, not generated address meshes.
Regenerate older libraries from their source; this tool has no older-format
decoder. Both constructions build only the live triangle prefix for each part.
The library digest changes with the new payload, so regenerate any populations
bound to that library as well.

Build the native viewer with `grown-review`, then use a fresh process and
output directory:

```powershell
cargo build --locked -p adventuresim-tactical-client `
  --bin tactical-scene-viewer --features grown-review

# Use the executable in the configured Cargo target directory.
tactical-scene-viewer --scene-input scene.json `
  --grown-review-library library.forest --grown-review-source SOURCE_ID `
  --grown-review-role candidate --grown-review-level full `
  --grown-review-batch-control ordinary `
  --output shared-plates
```

The five review options and scene input are required together. They conflict
with explicit fixtures, profiles, source authoring, benchmark modes and scene
overrides. Use `--view grown-middle-interior` to select one intended-distance
plate, or omit `--view` to capture the finite catalog. Warmup is implicit.

Every level and encoding uses the same source-only camera catalog. The
`grown-shared` close plate and warmup fit the union of the three original
source envelopes. The other five plates use each original source envelope's
projected radius, with the renderer's physical-pixel projection and level
selection policy. Candidate mesh bounds never determine framing.

| Plate | Target largest source radius |
|---|---|
| `grown-full-side` | 1.05 times the source's Full threshold |
| `grown-middle-full-side` | 0.80 times the Full threshold |
| `grown-middle-interior` | Geometric mean of Full and Middle thresholds |
| `grown-middle-coarse-side` | 1.05 times the Middle threshold |
| `grown-coarse-side` | 0.80 times the Middle threshold |

The factors position review plates around the existing transitions; they are
not new runtime thresholds. The report records all three roots' actual source
proxies and the level each would select from every previous level. Roots can
straddle a transition. Static plates do not demonstrate adaptive movement or
hysteresis over time. A supported camera can still have obstructed sight lines;
inspect the actual image before accepting its usefulness.

Cameras resolve once after the captured natural scene and its vista support
are admitted. Every eye uses the presentation owner's stitched terrain height,
with standing-eye clearance. Unsupported eyes and missing retained terrain
rings fail startup. Intended-distance plates display actual vista terrain.
The close plate remains a local geometry diagnostic. This bounded witness
uses natural terrain support; urban support is not admitted.

Observation and final screenshot checks retain the actual scene entity,
environment, vista revision, chunk membership and final eye-height samples.
They also require the actual physical viewport to match the fitting viewport.
These checks bind camera support; they do not seal every terrain asset byte or
prove that a terrain draw completed. Separate material-readiness and ordinary
GPU membership checks cover the visible terrain submissions.

The candidate uses real positions, normals and texture coordinates authored on
a separate compute device. The reference role evaluates original integer vertex
addresses in the vertex shader. Both share branch/leaf surface functions and
tactical lighting. Use `--grown-review-role reference` for that diagnostic
counterpart with the same level. Its meshes are built directly from checked
integer addresses and live triangle prefixes, with placeholder position,
normal and texture-coordinate channels. It does not create a compute device or
expand attributes only to discard them. The ordinary geometry and materials
are available with the environment crate's `prepared` feature on native and
Wasm targets; native compute expansion additionally needs `grown-authoring`.
Compiling for Wasm alone does not demonstrate browser rendering.

Geometry owns its actual vertex encoding and immutable selected level;
installation cannot choose a different interpretation. Materials derive their
encoding and record-address stride from that owner. Part identity and
the final capture checks include the level. Each level can partition its source
records differently under the allocation limit, so part counts can differ.

Fewer branch spans straighten the sampled curve. Fewer leaf blades change both
attachment and blade size. This preserves the existing renderer's approximation
policy; it does not establish visual suitability at a given distance. Compare
reference and candidate at the same level and camera for conversion fidelity,
then compare adjacent levels at the same intended-distance camera for quality.
The current close plate is a geometry diagnostic, not distant LOD acceptance.

Both constructors reject meshes above explicit per-part and total payload
limits before allocating their channels. Native expansion also applies its
device's compute/storage limits. Its device is released before the tactical
renderer starts. The two stages may
select different adapters or backends; inspect both receipts. Do not borrow an
active renderer for blocking authoring readback. Installation errors are
terminal; partially installed assets are not reusable results and the tool
supplies no replacement-source fallback.

## Scene and ownership

The destination roots use authored offsets at the scene centre and twelve metres
to either side. Each samples its own terrain support and retains identity scale
and yaw. Source growth on other terrain is not adapted to the destination slope.
Unrelated scene obstacles, playable ground scatter, vista trees, vista rock
and grass scatter, and named review specimens are removed. Shared playable,
detail and vista terrain remain. This isolated layout is not a complete forest
scene.

Upload the admitted bank once per presenting Bevy asset world with
`GrownBankAssets::upload`. Retain its returned `Arc` and pass it to every
`GrownAssets::install` for that bank, across models, levels and encodings.
Each installation checks the actual library identity and asset world before
creating its meshes or materials. Source handles are shared through this
explicit owner; uploading a second owner deliberately creates separate assets.
There is no automatic global lookup or replacement-source fallback.

`GrownAssets` retains that shared bank owner, source identity, mesh partitions,
selected level and ordinary materials. Placement identity
is separate from botanical source identity. Source root coordinates are removed
during library authoring, then destination root transforms are applied once. Do
not apply collider half-height normalization to these meshes.

Attach supplied bounds with `NoAutoAabb`. Address evaluation needs its original
source-conservative bounds; the candidate uses actual mesh bounds. Normal
frustum culling remains enabled. The positive control leaves automatic batching
enabled for tree parts. Shared terrain is deliberately unbatchable so its
individual material commands remain identifiable.

Meshes and source-buffer uploads use render-world extraction. The tool does not
retain diagnostic CPU mesh copies or compare full vertex arrays every frame.
The admitted bank remains retained; this does not establish physical RAM or GPU
memory use. Both materials provide forward and prepass shader paths. Analytic
leaf outlines retain fractional forward multisample coverage. Depth and shadow
fragments discard source alpha below 0.02 and otherwise write binary coverage.
This explicit cutout policy still needs context-specific visual review.

## GPU proof and rejection controls

The observer wraps successful main-pass material commands and snapshots actual
indirect arguments and compacted GPU mesh-instance data. Every shared tree part
must appear in exactly one command with all three intended instances. Tags join
actual GPU rows to distinct root entities, exact transforms, mesh allocations
and material slots. Three separate successful draws cannot pass. Visible terrain
must also match its held command bindings. The camera, inventory, tags and
renderer bindings remain held through the final screenshot callback.

The required member set does not shrink to match surviving GPU output. Use
explicit controls in separate fresh processes:

- `forced-singletons` disables automatic batching for the tree parts. It must
  fail the shared-command requirement.
- `hidden-last-root` hides the third root after warmup. It must fail the full
  three-root GPU membership requirement.
- `missing-last-tag` removes a held part's tag after warmup. It must fail the
  tag ownership check.
- `changed-level` changes the review selection after warmup while keeping the
  installed geometry and materials. It must fail the held source/level check
  before the next pose can authorize a GPU proof.

`grown-runtime.json` records the render adapter and depth planes.
`grown-review.json` records source metadata, held cameras, complete owned
inventory and GPU membership results. Raw snapshots, command receipts and proofs
are saved under `gpu-batches/`. The manifest records the actual fixed-level
capability, separate from adaptive gameplay tree presentation.

The manifest records equal settled raw-readback hashes. Normal output saves the
final RGB PNG; a settling mismatch can also save retry PNGs. It does not
guarantee independently decodable files for the accepted raw pair or retain
alpha. Compare reference and candidate images directly and report differences
rather than assuming pixel parity.

Diagnostic copies, readbacks and serialization add work. These runs are untimed
correctness evidence. Main-color membership does not validate shadow or prepass
membership, GPU vertex/source-buffer bytes, exact screenshot-frame timestamps,
moving cameras, browser behavior, adaptive detail or complete visible-area
arrival. Performance runs must measure their complete lifecycle separately.
