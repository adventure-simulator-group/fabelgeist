Walk/run blending passed a raw float through shared speed and cycle
interpolation, then admitted the same factor again for flight height.
Both interpolation methods now accept the existing `UnitInterval` type.
Locomotion admits its run factor once and reuses it across those operations;
the low-endurance speed producer also supplies an admitted blend.

The endurance smoothstep result is clamped to its intended `[0, 1]` range
before admission. `SpeedError` derives its error implementation and carries
`FractionError`; animation uses its existing fraction-error conversion.
Related: #780 and #726.

## Validation

Hosted checks are pending for the final revision. The workflow now executes the four existing
movement-speed tests and rejects empty or failing results.
No tests, cases, helpers, assertions or doctests were added.

## Implementation context

The shared core `UnitInterval` remains the sole fraction owner. Scalar values
are extracted inside the interpolation kernels and at the complementary
bounce calculation. This completes both speed callers and the cycle caller
without a raw-float overload. The catalog build script compiles the same
fraction owner and error derive as the runtime crate.
