Walk/run blending repeatedly extracted speed and length scalars across profile
selection, pose weights and authored stride calibration. Height compensation
then reconstructed the run fraction by dividing flight heights, which failed
when the configured run flight amplitude was zero. These consumers now carry
checked fractions directly from typed speed normalization through interpolation,
weight composition and height scaling.

`NonnegativeSpeed` owns clamped inverse interpolation with strictly increasing
bounds. The existing `UnitInterval` owns complements, products and smoothstep;
its cubic operation returns a bounded fraction without an output clamp. Shared
speed, cycle and length interpolation accept typed endpoints and fractions.
Scalar extraction occurs inside their numerical kernels, including the direct
Bevy/glam interpolation call.

Locomotion configuration now rejects run reference speeds at or below walk
reference speed, and blend speeds at or below the initial locomotion speed.
Its records and validation live together in a focused module. Zero flight
amplitude remains valid; authored height compensation uses the speed-derived
run fraction directly. Related: #780 and #726.

## Validation

Hosted validation pending for the review revision. No tests, cases, helpers,
assertions or doctests were added.

## Implementation context

The shared core `UnitInterval` remains the sole fraction owner, including in
the catalog build script. The connected changes cover all four run-fraction
producers, ordinary step distance, authored stride measurements, pose blend
weights, airborne travel and procedural height fades. Generic blend weights
retain their nonnegative range, including values above one.
