Walk/run blending passed raw speeds and lengths through gait profiles, pose
weights and measured stride distances. Height compensation reconstructed the run
fraction by dividing flight heights, which failed when the configured run flight
amplitude was zero. These paths now carry checked speed, length and fraction
values through their producers and consumers.

Speed interpolation and normalization compose typed subtraction, scaling,
addition and division. Addition returns a signed speed through its arithmetic
constructor, rejecting overflow. Interpolation uses the unit fraction guarantee
to construct its nonnegative result directly. Division checks its result, and
normalization admits the resulting ratio as `UnitInterval`. Positive speeds
widen through standard conversions. `UnitInterval` owns complements, products,
endpoint interpolation, interval normalization and the direct cubic smoothstep
with checked result admission.

Dive timeline production, launch/contact pose spans, procedural pelvis influence
and root-facing recovery use the same contact endpoint. Root handoff uses
validated fraction bounds, with its airborne endpoint authored in the combat
YAML. Span resolution keeps progress typed through its complement and conversion
to blend weights. Retained playback contributions stay typed through the
receiving calculations to the native vector and quaternion operations.

The zero-speed step-distance endpoint is a required positive-length field in
`content/tactical/combat.yaml`, carried by the server's complete configuration
snapshot. YAML-only changes trigger the existing quality workflow. Locomotion
configuration validates increasing walk/run reference speeds and initial/blend
speeds. Locomotion and state-transition tuning own their records and validation
in focused modules. Zero flight amplitude remains valid because height
compensation consumes the speed-derived run fraction directly. Related: #780 and
#726.

## Validation

Hosted validation is pending for this review revision. No tests, test cases,
test helpers, assertions or doctests were added. The existing gait test helper
was migrated to the canonical smoothstep operation after adapter removal.
Existing dive fixtures use typed influence values after removing the raw-factor
adapter.

## Implementation context

The shared core `UnitInterval` remains the sole bounded-fraction owner. Speed
division reuses the existing unrestricted `Ratio` unit; its result is checked
before bounded admission. Generic blend weights retain their nonnegative range,
including values above one. Native arithmetic boundaries include scalar
interpolation, sample-count normalization, weighted pose reduction and Bevy/glam
vector and quaternion operations. The catalog build script compiles the same
canonical owners as the runtime.
