# Constraint graph and particle contracts

`ParticleIndex` addresses solver state. `ConstraintIndex` addresses a record in
producer order; `ConstraintSlot` addresses its position after color permutation.
These identities cannot substitute for one another. An address assigns a role;
it does not prove membership in a particular particle buffer.

`ConstraintIncidence` owns complete fixed-arity records with a nonzero
`ConstraintArity`. Native flat and array constructors admit exact `u32` words
before coloring. Partial records fail with `ConstraintLayoutError`; no public
mutable representation can bypass that admission. `ConstraintEdges` owns the
two-particle records for distance and spring constraints. Shell geometry enters
these owners before its constraint-set construction calls.

Variable-arity coloring also accepts empty records and repeated addresses. It
preserves the original greedy candidate selection, arrival order within a color,
and native count narrowing. Private order and range tables prevent callers from
changing the permutation independently of its color intervals. `ColorRange`
retains nominal slots through sequential dispatch and encodes its first/count
uniforms at the GPU parameter boundary.

`ConstraintCount` is the shared per-constraint record cardinality used for graph
sizes, lambda allocation, attachment admission, and diagnostics. Empty particle,
lambda, and attachment bindings retain one zero word. No invocation dispatches
over an empty set. Particle words flatten only inside the owning upload adapter.

`ConstraintName` owns a set's human-facing diagnostic label. `PassParameterName`
owns shader lookup identity; the two roles cannot substitute for each other.
Parameter identity stays nominal through attachment replacement and dispatch.
Replacement retains the original policy: remove the previous resource and append
the new resource at the end of binding insertion order.

`ConstraintAttachment::from_records` is the native `NoUninit` serialization
boundary. It captures a borrowed payload together with `ConstraintCount`. Each
element is one complete per-constraint record, even when that record contains
several shader words. Reorder records with the set's graph-color permutation
before attaching them. `ConstraintSet::attach` admits exactly one record per
constraint; partial flattened records no longer form a second API.

An empty record collection keeps the original single zero-word binding,
independent of record width. A nonempty collection of zero-sized elements keeps
its original zero-byte allocation failure. Do not choose the empty-set sentinel
from byte occupancy alone. A native record's layout is its producer's contract;
attachment admission checks cardinality rather than proving shader reflection
compatibility.

`ConstraintLayoutError` distinguishes zero arity from incomplete records.
`ConstraintBuildError` retains set identity, record-count mismatches, the failed
buffer or kernel role, and concrete allocation, cache, or attachment causes.
`ConstraintDispatchError` retains set identity, clear/solve stage, color
interval, and its concrete provider cause. The boxed dispatch cause remains
classifiable. `ConstraintAttachmentError` retains parameter identity and nominal
actual/expected counts. Format these errors at presentation boundaries; inspect
variants and sources rather than classifying diagnostic prose.

## Particle state

`ParticleCount` counts active GPU records. `ParticleCapacity` counts physical
records and admits at least one sentinel, including an empty allocation request.
`ParticleInputCount` retains the full host position/adjacency cardinality before
the existing native unsigned narrowing; `InverseMassCount` gives the parallel
mass input its diagnostic role. Input admission checks mass agreement before
capacity, and position replacement checks the existing active count first.

Counts stay nominal through solver and collision dispatch, shell queries,
readback, and sorting handoffs. The owner encodes the existing sixteen-byte
particle record extent, item invocations, and uniform count at native
boundaries. Host slice indexing uses its explicit standard conversion. No
primitive count forwarding overload remains. Capacity does not prove buffer
membership or provider limits; native narrowing and provider admission remain
unchanged.

`ParticleError` distinguishes allocation, input lengths, capacity, replacement
count, and readback. Allocation and readback retain their buffer role and
concrete provider cause; diagnostics keep the original spelling. Shell reset and
position readback preserve this error directly. Failed input admission leaves
the previous state intact. Successful writes retain packed position/mass words,
reset active velocities, and leave the unused allocation tail unchanged.

`ParticlePositions` retains complete `ParticlePositionRecord` elements, with xyz
position followed by inverse mass. `ParticleVelocities` retains separate
`ParticleVelocityRecord` elements, with xyz velocity followed by an unused word.
Velocity construction clears that word. Both records preserve the existing
sixteen-byte ABI and private fields; upload encoding belongs to these owners.
The flattened `pack` and `unpack` APIs are removed.

Position-record construction rejects a parallel mass-count mismatch instead of
truncating a zip. Typed provider readback admits whole records, so a buffer with
a partial final record fails with the existing concrete `ReadbackError`. A
twenty-byte buffer contains five scalar words but does not contain a complete
array of particle records. Valid state allocations and their native words are
unchanged. Native record admission does not establish mass validity or buffer
membership; physical admission remains an explicit CPU operation.

Triangle incidence, host mesh query quantities, remaining material quantities,
other host timing, flags, and generic per-record permutation remain debt.
Native upload and construction ports do not exempt whole mathematical modules,
geometry producers, test helpers, or generic internal operations.

## Particle units and corrections

`ParticleMass` carries kilograms. `ParticleInverseMass` carries inverse
kilograms, and `ParticleArealDensity` carries kilograms per square metre.
Triangle area contributes one third of its mass to each corner in arrival order.
The massless policy keeps the original strict threshold and returns positive
zero for a massless vertex. Density stays nominal through fabric presets,
garment assembly, fitted garments and shell construction. Mass and inverse mass
cannot substitute for one another.

Native inverse-mass construction preserves every float word, including signed
zero and nonfinite words. CPU surface admission requires finite nonnegative
values. `ParticleMobility` distinguishes prescribed motion from dynamic
response; both signs of zero remain prescribed. Admission does not rewrite GPU
state. Position records retain the inverse-mass owner through native encoding,
readback, host caching and correction.

`ConstraintGradient` carries a signed separation derivative. `BarycentricWeight`
carries an affine triangle coordinate; those roles cannot substitute for one
another. Contact response preserves mass times gradient times gradient. Layer
response preserves squared barycentric weight times mass. Their distinct
correction operations retain the original multiplication and division grouping.
`EffectiveInverseMass` owns each response threshold and accumulation. Position
corrections and normal-speed corrections carry separate units through vector
application. Clipping retains its affine weights and interpolation fraction.

The CPU distance oracle retains typed violation, compliance per squared substep,
and accumulated multiplier. Its distance algorithm remains independent of the
GPU shader. Unit arithmetic owns scalar operations without exposing a forwarding
API that unwraps units into another handwritten primitive helper.

Private `native_sum_term` conversions feed only the standard floating-point
accumulator and immediately retain the result's unit. Keep the original standard
sum, including its negative-zero identity and native numerical behavior. These
specific conversion ports do not exempt the rest of the mathematical code.
Other vector, geometry, timing, index and material interfaces remain work.

Forty native triangle-mass snapshots retain per-vertex mass words, the massless
inverse policy and the standard total. They cover degenerate, tiny and nonfinite
geometry, signed zero, threshold behavior and nonfinite density. These snapshots
preserve native construction behavior separately from CPU physical admission.

`ParticleInputIndex` preserves the full host address before count admission;
it cannot substitute for a native GPU `ParticleIndex`. `ParticleMassCount`
retains kilogram-input cardinality separately from `InverseMassCount`. Shell
admission uses these owners for diagnostic context without narrowing malformed
inputs or treating a diagnostic address as proof of array membership.

## Solver scheduling and failures

`StepDuration` carries a complete driver's seconds; `SubstepDuration` carries
one prediction/constraint/finalization interval. The frame owner performs
subdivision. `SubstepCount`, `SubstepIndex`, and `ConstraintSweepCount` retain
separate scheduling roles. Zero solver substeps skip a full step; zero sweeps
still perform one sweep. The shell's interactive driver retains its existing
minimum of one substep and its independent finite-positive admission policy.
Do not strengthen either admission policy while converting interfaces.

`GravityAcceleration`, `DampingRate`, and `ParticleSpeedLimit` retain their
units through settings, CPU integration and native uniform binding.
`ConstraintCompliance` owns division by the squared substep. Native constructors
retain float words, including nonfinite values and signed zero; they do not
promise physical validity. Preserve operation grouping and native shader words.
Seconds and counts encode only at their owning uniform, capture or serialized
settings boundaries. Internal consumers use unit operations rather than scalar
forwarding helpers.

`SubstepHook` retains an associated concrete error type. `NoSubstepHook` carries
an impossible failure; a closure may return its own concrete error.
Solver entry points borrow `dyn SubstepHook<Error = E>` so a previously selected
participant can enter the same interface. Only its concrete error carrier is
generic. The borrowed participant uses dynamic dispatch without allocation;
its lifetime remains independent of the error's required static lifetime.
`HookChain::new(first, second)` composes ordered typed participants and retains
which participant failed. Its type parameters are the participants' concrete
error carriers, and its fields borrow their domain interfaces. Nest chains to
compose more participants. The chain
preserves its established record-only behavior: its default `after_solve` does
not forward to participants. The shell's separate hook owns its before/after
body and self-collision ordering.

`SolverBuildError` retains shader preparation context. `SolverDispatchError`
retains predict/finalize role and the concrete dispatch cause.
`SolverSubstepError` distinguishes shader dispatch, constraint dispatch and hook
phase; `SolverStepError` adds the failing nominal substep. The generic hook
cause remains statically typed through those structured errors. Standard
`Error::source` erases it only for external cause inspection. Shell, cloth and
fit stepping retain `ShellStepError`, including interval reads and host
projection failures. Generic presentation errors remain debt at other callers.
