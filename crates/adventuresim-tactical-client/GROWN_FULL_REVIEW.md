# Review shared Full tree meshes

The native tactical viewer places three grounded instances of one admitted
source tree. It installs the Full mesh and material closure once, then shares
the handles between placements. The diagnostic checks actual indirect GPU draws
and compacted instance records before saving an image. Full means the library's
declared detail, held fixed throughout the run.

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
decoder. Full authoring builds only the live triangle prefix for each part.
The library digest changes with the new payload, so regenerate any populations
bound to that library as well.

Build the native viewer with `grown-full-review`, then use a fresh process and
output directory:

```powershell
cargo build --locked -p adventuresim-tactical-client `
  --bin tactical-scene-viewer --features grown-full-review

# Use the executable in the configured Cargo target directory.
tactical-scene-viewer --scene-input scene.json `
  --grown-review-library library.forest --grown-review-source SOURCE_ID `
  --grown-review-role candidate --grown-review-batch-control ordinary `
  --output shared-plates
```

The four review options and scene input are required together. They conflict
with explicit fixtures, profiles, source authoring, benchmark modes and scene
overrides. The `grown-shared` plate and its warmup use the same camera, derived
from the union of the three original source envelopes. Candidate mesh bounds
never determine framing. A supported camera can still have obstructed sight
lines; inspect the actual image before accepting its usefulness.

The candidate uses real positions, normals and texture coordinates authored on
a separate compute device. The reference role evaluates original integer vertex
addresses in the vertex shader. Both share branch/leaf surface functions and
tactical lighting. Use `--grown-review-role reference` for that diagnostic
counterpart; its GPU membership requirement remains unchanged.

The author rejects meshes above explicit per-part and total payload limits. Its
device is released before the tactical renderer starts. The two stages may
select different adapters or backends; inspect both receipts. Do not borrow an
active renderer for blocking authoring readback. Installation errors are
terminal; partially installed assets are not reusable results and the tool
supplies no replacement-source fallback.

## Scene and ownership

The destination roots use authored offsets at the scene centre and twelve metres
to either side. Each samples its own terrain support and retains identity scale
and yaw. Source growth on other terrain is not adapted to the destination slope.
Unrelated scene obstacles, playable ground scatter, vista trees and named review
specimens are removed. Shared terrain remains; vista rock and grass background
can remain too. This isolated layout is not a complete forest scene.

`GrownFullAssets` retains the admitted bank, source identity, mesh partitions,
source-buffer handles and ordinary materials. Placement identity is separate
from botanical source identity. Source root coordinates are removed during
library authoring, then destination root transforms are applied once. Do not
apply collider half-height normalization to these meshes.

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

`grown-full-runtime.json` records the render adapter and depth planes.
`grown-full-review.json` records source metadata, held cameras, complete owned
inventory and GPU membership results. Raw snapshots, command receipts and proofs
are saved under `gpu-batches/`. The manifest records fixed Full capability,
separate from adaptive gameplay tree presentation.

The manifest records equal settled raw-readback hashes. Normal output saves the
final RGB PNG; a settling mismatch can also save retry PNGs. It does not
guarantee independently decodable files for the accepted raw pair or retain
alpha. Compare reference and candidate images directly and report differences
rather than
assuming pixel parity.

Diagnostic copies, readbacks and serialization add work. These runs are untimed
correctness evidence. Main-color membership does not validate shadow or prepass
membership, GPU vertex/source-buffer bytes, exact screenshot-frame timestamps,
moving cameras, browser behavior, adaptive detail or complete visible-area
arrival. Performance runs must measure their complete lifecycle separately.
