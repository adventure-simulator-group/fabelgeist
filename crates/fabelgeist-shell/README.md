# Bending records

`BendPoints` admits four positions ordered as the two hinge endpoints and then
the two opposite triangle vertices. That order must match `BendQuad::particles`.
`BendWeights::for_points` finds the scaled affine coefficients and returns a
`BendGeometryError` identifying the first rejected numerical stage. Producers
skip rejected hinges. Thresholds, arithmetic order, and nonfinite-input behavior
are unchanged; an error stage does not diagnose every possible geometric cause.

Weights own rest capture. `observed_rest` measures the supplied geometry using
the ascending, zero-start vector fold. `flat_rest` supplies a sewn hinge's zero
target, and `BendRecord::slack` disables a hinge during seam closure. The scaled
coefficients have length units; the resulting rest measure has squared-length
units and is distinct from a stretch rest length or compliance.

`BendRecord` privately owns the four coefficients, one rest measure, and three
zero padding words in a 32-byte native GPU record. Shell meshes, cloth panels,
fitted garments, and seam closure retain whole records. Reorder them with the
constraint set before passing them to its native `attach` operation. Never
flatten or upload bare coefficients: the kernel consumes one complete record
per hinge. Empty sets retain the existing unused single zero word.

The focused tests compare 36 frozen original-algorithm cases by their native
words, including folds, scales, degeneracy, signed zero, and nonfinite input.
GPU readback checks cover colour permutation, replacement, rejected partial
collections, padding, and the empty binding. These checks do not establish
visual fitting quality or resolution-independent material calibration.

General constraint identities, parameter bindings, material values, topology
addresses, and solver errors remain separate bespoke-type migration work.
