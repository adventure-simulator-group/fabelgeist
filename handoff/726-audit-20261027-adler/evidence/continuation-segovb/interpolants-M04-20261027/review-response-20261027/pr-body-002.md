Walk/run blending passed raw speeds and lengths through gait profiles, pose
weights and measured stride distances. Height compensation reconstructed the
run fraction by dividing flight heights, which failed when the configured run
flight amplitude was zero. These paths now carry checked speed, length and
fraction values through their producers and consumers.

Speed interpolation and normalization compose typed subtraction, scaling,
addition and division. Addition and division check their results; normalization
admits the resulting ratio as `UnitInterval`. Positive speeds widen through
standard conversions, and normalization accepts typed bounds. `UnitInterval`
owns complements, products and the direct cubic smoothstep with checked result
admission. Phase consumers call that operation directly.

The zero-speed step-distance endpoint is a required positive-length field in
`content/tactical/combat.yaml`, carried by the server's complete configuration
snapshot. YAML-only changes now trigger the existing quality workflow.
Locomotion configuration validates increasing walk/run reference speeds and
initial/blend speeds. Its records and validation live in a focused module.
Zero flight amplitude remains valid because height compensation consumes the
speed-derived run fraction directly. Related: #780 and #726.

## Validation

Hosted validation is pending for this review revision. No tests, cases,
helpers, assertions or doctests were added. The existing gait test helper was
migrated to the canonical smoothstep operation after adapter removal.

## Implementation context

The shared core `UnitInterval` remains the sole bounded-fraction owner.
Speed division reuses the existing unrestricted `Ratio` unit; its result is
checked before bounded admission. Generic blend weights retain their
nonnegative range, including values above one. Scalar extraction occurs inside
owning numerical kernels, including the direct Bevy/glam interpolation call.
The catalog build script compiles the same canonical owners as the runtime.
