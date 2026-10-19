# Review source trees as ordinary Full meshes

The native tactical viewer compares two evaluations of one admitted tree
library. The reference expands each original integer vertex address in the
vertex shader. The candidate uses positions, normals and texture coordinates
authored from those same records on a separate compute device. Both use the
same branch and leaf surface functions and the tactical presentation's lighting.
Full means the library's declared detail, held fixed throughout the capture.

This is a static content diagnostic. It does not replace the gameplay tree
presenter, select adaptive levels of detail, or demonstrate arrival performance.

## Input and commands

Supply a checked `library.forest`, its actual botanical `PlantId`, and a
tactical scene input. The identifier is not a model ordinal or a placement ID.
The selected model must contain both branch and leaf records. Keep the source's
generation evidence separately: decoding the library cannot establish how its
records were produced, its biological age, or species calibration.

Build the native viewer with `grown-full-review`, then run each role in a fresh
process and output directory:

```powershell
cargo build --locked -p adventuresim-tactical-client `
  --bin tactical-scene-viewer --features grown-full-review

# Use the executable in the configured Cargo target directory.
tactical-scene-viewer --scene-input scene.json `
  --grown-review-library library.forest --grown-review-source SOURCE_ID `
  --grown-review-role reference --output reference-plates

tactical-scene-viewer --scene-input scene.json `
  --grown-review-library library.forest --grown-review-source SOURCE_ID `
  --grown-review-role candidate --output candidate-plates
```

The three review options and scene input are required together. They conflict
with explicit fixtures, profiles, source authoring, benchmark modes and scene
overrides. `--view grown-root`, `--view grown-crown` and
`--view grown-gameplay` can select individual plates. Every run includes a
pipeline warmup. Without `--view`, it captures all three plates.

The native author rejects meshes above the explicit per-part and total payload
limits. Its device is released before the tactical renderer starts. Authoring
and rendering may select different adapters or backends; inspect both receipts.
Do not borrow the active renderer for blocking authoring readback. An asset
installation error is terminal: partially installed assets are not a reusable
result, and the tool supplies no replacement-source fallback.

## Scene and materials

Both roles retain the actual scene terrain, lighting, exposure and depth planes.
Cameras derive from the original source bounds and destination terrain, never
from the candidate's fitted mesh bounds. Terrain support at the camera does not
guarantee an unobstructed view. Inspect the actual images before accepting their
usefulness for comparison.

The source root is translated to terrain height at the scene origin, with
identity scale and yaw. Growth recorded on different terrain is not adapted to
the destination slope. Unrelated scene obstacles, playable ground scatter,
vista trees and named review specimens are removed. Shared terrain remains;
vista rock and grass background can remain too. This isolated specimen is not
a complete forest scene.

`GrownFullAssets` retains the admitted bank, source identity, mesh partitions,
source buffers and ordinary wood/leaf materials. Address evaluation requires the
source-conservative bounds. Attach the supplied bounds with `NoAutoAabb`, so
Bevy does not replace them from the reference mesh's empty position channels.
The candidate uses its actual mesh bounds. Normal frustum culling remains on.

Both materials provide forward and prepass shader paths. Leaf outlines use the
source analytic shape. Forward color retains fractional multisample coverage;
depth and shadow fragments discard source alpha below 0.02 and otherwise write
binary coverage. This is an explicit shadow-cutout policy. Static images do not
validate moving cameras, motion vectors, every shadow pass, or another lighting
context.

## Evidence and limits

`grown-full-runtime.json` records the actual render adapter and depth planes.
`grown-full-review.json` records original source metadata, held cameras, full
part coverage, owned inventory and successful main-pass material commands.
`manifest.json` distinguishes this fixed Full capability from adaptive tree
presentation. One owned source root is checked independently of the ordinary
gameplay tree markers.

Owned meshes and shared terrain are deliberately unbatchable for this content
check. The screenshot requires actual successful commands for the visible
owned parts and terrain, with held bindings and a completed queue prefix. The
callback checks the source inventory again. A queue prefix does not identify
the screenshot's exact GPU frame. CPU asset snapshots and upload-lifecycle
checks do not read back GPU storage bytes or count shadow draws.

The manifest records equal settled raw-readback hashes. Normal output saves
the final RGB PNG for each view; a settling mismatch can also save retry PNGs.
The tool does not guarantee independently decodable files for the accepted raw
readback pair or retain its alpha channel. Compare the saved reference and
candidate images directly and report differences rather than assuming parity.

Retained CPU meshes, immutable-value snapshots and inspection work change the
resource cost. These captures do not measure production memory or performance,
prove multi-tree GPU instancing, or establish complete visible-area arrival.
