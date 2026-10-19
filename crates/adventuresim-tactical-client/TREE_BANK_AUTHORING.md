# Mature tree bank authoring

The native scene viewer can convert the existing mature English oak and common
beech recipes into checked prepared-forest assets without starting a renderer
or a growth simulation. Enable the `foliage-review` feature and provide an
actual tactical scene input:

```text
cargo run --locked -p adventuresim-tactical-client \
  --bin tactical-scene-viewer --features foliage-review -- \
  --scene-input path/to/scene.json --prepare-tree-bank target/tree-bank
```

The parent output directory must exist. The bank directory must be fresh; the
tool rejects an existing directory. Use this authoring invocation separately
from capture options. A failure may leave an incomplete fresh directory.
`TREE_BANK_OUTPUT` is printed only after all four files have been written.

The tool uses the admitted scene environment and canopy competition, four
existing deterministic specimen seeds per species, and the current source leaf
state. The four beech seeds are variations of the existing beech recipe. They
are not four distinct calibrated gnarling recipes. These are mature procedural
sources, with no simulated chronological growth or strategic season claim.

The outputs are `unique.forest`, `library.forest`, `population.forest`, and
`source.json`. The library retains all eight source specimens. Source model
identities remain separate from placement identities. Population placement uses
the current required visibility format; there is no older-format decoder.
Source geometry moves from collider-centred coordinates to botanical root zero
once, by half the production trunk height.

The report records input and generated scene digests, actual source recipes and
seeds, source centerlines, logical leaf rectangles and petioles, static generated
flat-card and cambered-card mesh envelopes, and each converted model's identity,
record spans and bounds. Mesh envelopes precede animated wind and do not clip
transparent pixels. The authored flat floor is isolated review geometry, not
the supplied scene's terrain.

Conversion approximates source leaf-box area with analytic shoot clusters.
Individual card transforms, production material response, transported branch
frames, root aspect and lift, collars, taper and rounded caps are not all
preserved. Successful authoring establishes checked asset construction, not
matching appearance, runtime replacement, or arrival performance. Production
use requires matched images under actual game lighting and separate arrival
measurements.

## Matched static content review

Use the same scene input to render one source specimen or its converted bank
model under tactical production lighting and terrain:

```text
tactical-scene-viewer --scene-input path/to/scene.json \
  --bank-review-dir target/tree-bank --bank-review-source SOURCE_ID \
  --bank-review-role source --output target/source-plates
```

Read `SOURCE_ID` from `source.json`. Repeat with `--bank-review-role candidate`
and a different fresh output directory. The three review flags belong together
and require `--scene-input`; explicit profiles, fixtures, authoring options and
benchmark modes conflict with this review. Ordinary capture commands do not
require the review flags.

Each process admits the bank, source binding and complete scene environment.
Root, crown and eye-height cameras come from the full source geometry and are
shared by both roles. Candidate uses fixed full detail. Actual tactical terrain,
lighting, exposure and frozen clock phase remain; unrelated obstacles, scatter,
vista trees and old review specimens are removed. Vista terrain and rock bands
remain common background subject to the fixed views' visibility settings.

`bank-review-runtime.json` records the actual adapter and camera depth planes.
`bank-review.json` records source identity, camera contracts and successful
role-specific draw prerequisites through the final screenshot callback. Source
uses actual production wood, cambered leaves and buds; Candidate uses the bank
renderer. Each accepted view requires identical settled readbacks. Missing or
changed cameras, incomplete draws and failed readbacks reject completion.

These are held content plates. Draw prerequisites precede the screenshots and
do not timestamp their exact GPU frame. Automatic gates do not establish
appearance, botanical fidelity, moving LOD behavior or full-area arrival time.
Inspect the actual paired images before accepting a representation change.