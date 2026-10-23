# Canonical regional roads

The offline map compiler exports the same full active Viabundus and inferred
walking-link geometry used to build the final terrain road mask. Raster zoom
filtering and simplification do not participate in this source contract.

`just build-strategic-map` produces final terrain and
`target/strategic-map/regional-roads-v1.json` with
`target/strategic-map/regional-roads-v1.pack`. `just build-regional-roads`
rebuilds only the vector package against initialized Viabundus data, an existing
compiled world and its matching final terrain. Base inference terrain remains
a separate compiler input and is never accepted as a regional road source.

The manifest identifies the final terrain source, its canonical road geometry
digest and the binary content digest, with bounded line and point counts. Each
binary line begins with a classification byte, then a little-endian `u32`
point count, followed by pairs of
little-endian IEEE-754 `f64` longitude and latitude degrees. Source precision and
line ordering are preserved exactly. The package admits at most 50,000 lines
and 1,000,000 points, approximately 16 MB of binary coordinates.

Classification tags are land road (0), river shipping (1), coastal shipping
(2), canal shipping (3), ferry (4), winter route (5), and inferred walking link
(6). They remain distinct through runtime admission so presentation can
distinguish water crossings and conditional connections from land roads.
Classification does not change travel authority. The terrain geometry digest
covers the original coordinate arrays; the payload digest also covers tags.

`road_pack::RoadPack::load` verifies the file bound, final source identity,
payload hash, declared counts and canonical geometry digest against the terrain
road-mask digest. Only then does it produce checked geographic E7 positions,
with inclusive bounds for window selection. E7 positions use ten-millionths of
a degree; the conversion affects presentation only and never rewrites routing
geometry or the terrain mask.

Load the package on first road demand in blocking work and retain it for the
immutable source lifetime. Admission temporarily holds bounded source bytes
and geometry; the resident package holds checked positions. `intersecting`
selects complete lines whose bounds overlap a checked `Wgs84BoundsE7`. Consumers
must still clip and bound their presentation output. This is a source package,
not a scene generator, gameplay authority or renderer resource.

For read-only admission diagnostics, run:

```text
cargo run -p adventuresim-terrain --example inspect-road-pack -- TERRAIN_JSON TERRAIN_PACK ROAD_JSON ROAD_PACK
```

It reports the accepted source, full line and point counts, and admission time.

Runtime bundle schema 2 distributes these vector files and final terrain. It
omits the former AVIF map manifests and tile pack. Regenerate the runtime
release and its lock before distributing this bundle; schema-1 releases are
rejected. The release tool checks binary structure, content and terrain
coherence, while Rust admission owns the canonical geometry hash check.
