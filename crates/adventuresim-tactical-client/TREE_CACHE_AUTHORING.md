# Production tree cache authoring

The native foliage review tool can prepare one mature production tree variant
without opening a renderer. This preserves the existing ordered branch and
leaf records, mesh channels, triangle indices and near canopy atlas. It is a
content trial; the gameplay presenter does not yet consume these files.

Build the native viewer with the `foliage-review` feature, then select an actual
production source identifier from the source-bank authoring manifest:

```powershell
cargo build --locked -p adventuresim-tactical-client `
  --bin tactical-scene-viewer --features foliage-review

# Use the executable in the configured Cargo target directory.
tactical-scene-viewer --scene-input scene.json `
  --prepare-tree-cache fresh-cache --cache-source SOURCE_ID

tactical-scene-viewer --scene-input scene.json `
  --check-tree-cache fresh-cache/variant.treecache --cache-source SOURCE_ID
```

Authoring requires a fresh directory and writes `variant.treecache` plus
`cache.json`. The manifest records the input, environment, source identifier,
species, authored seed, production cache key, payload digest, byte sizes and
per-group vertex counts. Repeating authoring with identical inputs produces
identical files. An existing directory is rejected without replacement.

The closed payload contains the detailed and middle-distance trunk, detailed
wood, buds and cambered leaves for each actual primary crown group. It also
contains the first canopy atlas tier because the production Full construction
path uses that bake to obtain cluster bounds. This dependency is retained even
though Full would not draw that atlas. Other adaptive tiers, flat leaves and
aggregate wood are absent. This artifact cannot certify adaptive LOD behavior.

Coordinates remain collider-centred metres. A future presenter must apply the
existing half-trunk placement normalization exactly once. Trunk COLOR values
carry metric root height; leaf COLOR values carry shading, shoot-shadow
selectors and ambient visibility. They are preserved as authored, including
leaf brightness above one. Materials, asset handles, entities and
terrain-contact bindings are not serialized. They belong to the destination production presenter.

Admission checks the current format, complete BLAKE3 payload digest, exact
uncompressed length, compression termination, input/environment/source binding,
part schemas, finite channels, normals, indices, bounds, group layout and
atlas metadata. Collection lengths are bounded before reservation. The fixed
shape and collection limits establish a conservative allocation envelope for
decoding and structural validation; this is not measured process memory or a
whole-load bound. Native content validation additionally regenerates the
selected source once to compare its complete records and bud completeness.
That comparison is an offline correctness control, not an arrival fast path.

Successful CPU validation establishes serialization and admission only.
Production materials, shadows, actual cache attachment, held images, moving
LOD transitions, distant instancing and complete visible-area arrival require
separate runtime validation and performance evidence.
