# Particle state and mass

`ParticleMass` represents kilograms, `ParticleInverseMass` represents inverse
kilograms, and `ParticleArealDensity` represents kilograms per square metre.
Triangle mass accumulation preserves triangle arrival order. Massless vertices
remain pinned using the existing strict threshold.

Native inverse-mass construction preserves all float words. CPU admission uses
`MassValidity`, while `ParticleMobility` distinguishes prescribed from dynamic
motion. A prescribed collider may move between supplied positions; zero inverse
mass does not imply a stationary obstacle.

`ParticlePositions` pairs each position with its inverse mass. It rejects
mismatched parallel inputs before creating native records. `ParticleVelocities`
owns velocity records, with a zeroed unused shader word. Both expose borrowed
`BufferUpload` values; readback decodes complete records and limits results to
the active particle count. Position and velocity collections are separate
types, even though their native records both occupy four scalar words.

Contact gradients, barycentric weights, effective inverse mass, and projection
corrections retain distinct roles through the host solver. The checked native
word fixtures cover particle records, triangle mass accumulation, contact
resolution, and layer projection.

## Constraint graph addresses and records

`ParticleIndex` identifies a solver particle in graph incidence. A
`ConstraintIndex` refers to the producer's original record, while a
`ConstraintSlot` refers to its position after permutation. Immutable color
ranges keep those slots together for sequential solve dispatches; within a
color, distinct constraints cannot share a particle. Greedy arrival order and
stable counting sort determine the schedule.

`ConstraintEdges` owns two-particle records for distance and spring sets.
`ConstraintIncidence` admits complete fixed-arity records with nonzero arity;
malformed native inputs return `ConstraintLayoutError`. Variable-arity coloring
still accepts empty records and repeated addresses. The graph assigns roles and
conflicts, without proving buffer membership or agreement with a shader's arity.

`ConstraintCount` remains separate from `ColorCount` through construction,
attachment admission, lambda allocation and recording. Upload and shader
parameter adapters retain native integer widths and narrowing. Empty sets keep
one unused native word and schedule no dispatches. Generic attachment,
allocation and kernel errors remain separate migration work.

Native input constructors copy incidence into owned immutable host records.
This adds host storage during construction; no performance improvement is
claimed. Tests compare 309 frozen original schedules and native upload,
allocation, rejection and no-op behavior. Solver arithmetic and floating
aggregate operations remain unchanged.
