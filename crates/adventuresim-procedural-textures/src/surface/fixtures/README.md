# Oak height reference

`august-height.f32` contains 64 by 64 row-major, little-endian `f32` samples
from `oak_bark_height` at commit
`12d529d4151ae2f6f9faadf1cdf96127a173fe39`, the parent of the September 1
texture rewrite. Sample coordinates are `(x + 0.5, y + 0.5) / 64`.

The reference was evaluated independently from the historical implementation
in `crates/adventuresim-tactical-client/src/presentation/procedural_assets/mod.rs`,
using its original SplitMix64 hash and inclusive 24-bit unit conversion.
These are signed scalar heights before texture packing; multiply by 0.032
to obtain metres. The tile spans 0.5 metres in both directions.

This fixture protects the actual plate, crown, and fissure field, independently
of subsequent module layout, editable controls, or texture channel packing.
It is reference data, not an alternate generator or runtime fallback.
