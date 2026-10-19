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
