# Shared shell geometry

`BendPoints` keeps the two hinge endpoints followed by their opposite vertices.
That order must match `BendQuad::particles`, weight generation, and the GPU's
four-particle constraint record. `BendWeights::for_points` owns plane selection,
signed affine minors, normalization, and the existing local area scale.
`BendGeometryError` identifies collapsed spokes, collinear spokes, vanishing
minors, or negligible hinge area in their established admission order. These
variants identify failed algorithm stages rather than diagnosing nonfinite input
geometry. Producers skip these hinges just as they previously skipped absent
weights.

`BendWeights` retains its private coefficients through rest capture.
`BendRestMeasure` represents the magnitude of that scaled affine combination,
which differs from edge length and compliance. Its unit is squared length
because the weights include a length scale. Observed rest keeps the original
ascending, zero-start sum and Euclidean norm. A sewn hinge uses a flat rest
independent of its initial panel gap. A seam hinge held slack uses an all-zero
record, including its coefficients.

`BendRecord` owns the fixed eight-word native layout: four coefficients, rest,
and three zero padding words. Its representation is private, and `NoUninit`
permits exact serialization without allowing arbitrary bytes to construct a
record. Keep records nominal through garment assembly, graph-color permutation,
and constraint attachment. Do not flatten them into primitive arrays between
these owners. `BendRecordValidity` retains the existing finite-data check.

The original geometric thresholds, float arithmetic, weight scale, and shader
source remain unchanged. Executable fixtures from the original algorithm pin
signed, tiny, large, degenerate, and nonfinite inputs and the native record
bits. The model retains its documented resolution sensitivity; this type
migration does not recalibrate material parameters.

Triangle addresses, rest lengths, material scalars, collision state, and other
raw interfaces remain migration debt. Keep them visible in the inventory
until their complete producer and consumer chains have semantic owners. GPU
mechanics remain transient server state.

Shell construction admits native edge and hinge addresses into the shared XPBD
`ConstraintEdges` and `ConstraintIncidence` owners before building sets. The
graph retains particle identity through color ordering and upload. Raw triangle
and mesh-topology interfaces remain migration debt; these construction ports do
not exempt their producers or other geometric operations. See the [XPBD
contract](../fabelgeist-xpbd/README.md).

Shell particle queries and self-collision use the shared `ParticleCount` and
`ParticleCapacity` owners. Host adjacency construction retains
`ParticleInputCount`. Spatial hash layout owns its bucket size, byte extent,
clear dispatch, and parameter encoding; active counts pass directly into the
shared sorting owner. This preserves sentinel allocation and the existing
native hash sizing arithmetic.

`Shell::reset`, `Shell::read_positions`, `Shell::read_mesh` and
`Shell::position_validity` return `ParticleError` directly. Position validity
retains finite or nonfinite state as `ShellPositionValidity`; failed reads
remain errors. Solver-backed step interfaces still retain generic error
migration debt.

Host contact and outer-layer projection retain the shared `ParticlePositions`
and `ParticleVelocities` records through upload. Previous-position readback
uses complete position/mass records. Neither path flattens state into scalar
streams between handwritten helpers. Shared particle units retain mass and
inverse mass through CPU correction; other geometry interfaces remain work.

`SurfaceContactCount` counts successful positional resolutions across all solve
passes; a repeated pair may count again. Keep it nominal through pair
resolution, vertex and edge passes, host solving, GPU projection, and
presentation. It is independent of active particle count. Zero resolutions skip
state uploads.

`SurfaceProjectionError` distinguishes interval-count admission, previous-state
readback, and particle state operations. Particle failures retain the position
read, velocity read, or position-encoding stage and concrete `ParticleError`;
standard cause inspection retains the provider cause beneath it. Interval
mismatches keep full host and active GPU counts and fail before readback or
mutation. Outer-layer projection returns `ParticleError` directly. CCD
mathematics and remaining geometry quantities remain migration work.

Triangle mass construction consumes `ParticleArealDensity` and returns
`ParticleMass`; inverse-mass producers return `ParticleInverseMass`. Contact and
outer-layer consumers retain those units. A clipped sample owns its point and
barycentric coordinates rather than using a scalar tuple alias. Signed contact
gradients remain distinct from those affine coordinates. Preserve the original
response formulas, correction grouping, standard sums and clipping order.

Independent native snapshots pin 36 swept contact cases, 24 outer-layer cases
and 40 triangle-mass cases. The layer cases include overflow and nonfinite mass
arithmetic. Position, velocity, resolution/change state and residual words
remain exact. Native GPU snapshots retain signed zero, NaN and infinity through
upload and readback. No stronger physical validity is inferred from admission.

`ShellMeshError` retains invalid density, full host particle and mass counts,
nonfinite input addresses and values, constraint cardinalities, particle
reference roles and rest-data families. `from_mesh` retains the concrete
`MeshTriangleError`, including its validation cause. The original first failed
contract still determines the error: particle admission, constraint lengths,
triangle/edge/seam references, bend references, then rest data. Empty geometry
remains constructible and fails simulation admission.

`ParticleInputIndex` retains the full host address before GPU admission. It is
distinct from a native `ParticleIndex`; malformed host inputs must not truncate
diagnostic addresses. `ParticleMassCount` counts kilogram inputs and remains
distinct from inverse-mass cardinality. These shared diagnostic owners do not
admit an input or establish membership in a particle array.

`ShellMaterialError` retains the failed parameter. Finite nonnegative admission
checks stretch, bending, seams, thickness, friction and damping in that order;
the positive-thickness check follows all of them. Both signs of zero remain
valid for other parameters. Display retains the established messages while
callers classify variants and inspect concrete causes.

`SelfCollisionBuildError` distinguishes adjacency admission, kernel compilation,
sort preparation, buffer allocation and scratch allocation. Resource roles own
their shader selection, exact native buffer labels and error context.
`SelfCollisionRecordError` retains capacity, failed kernel dispatch and concrete
sort failures. Disabled and small passes keep their original admission order;
zero requests retain the same native sentinel allocations. Kernel selection,
buffer layout, dispatch order and numerical shader source remain unchanged.
Seven original GPU cases pin 21 states across fresh rebuild, reuse, pinned
particles, adjacency suppression and disabled passes.

`ShellBuildError` retains mesh, material, particles, constraint family, bending
preparation and self-collision causes. Mesh and material admission precede
device resources. The unreachable second empty-mesh check is removed; empty
meshes keep
their existing mesh admission error. Bending preparation retains color order and
complete weight records.

`ShellProjectionError` retains outer-layer particle state, residual-position
reads or concrete surface projection failures. Inspect these stages and the
cause chain instead of their display text. Display keeps the lower provider's
message.
The handwritten `SubstepHook` trait still mandates a generic result internally;
its adapters preserve concrete causes and remain explicit migration debt. That
trait is not an external-boundary exception.
