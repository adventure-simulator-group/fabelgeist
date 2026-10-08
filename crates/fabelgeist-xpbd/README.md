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

`ParticleCount` describes a native GPU particle prefix. `ParticleCapacity`
describes allocated records, including the one-record sentinel for an empty
request. A capacity cannot be passed as an active count. `ParticleInputCount`
keeps full host lengths distinct until an explicit native narrowing; that
narrowing truncates and does not validate device limits or allocation identity.

Counts remain bespoke types through construction, write admission, readback,
solver and collision parameters, and shell queries. Host garment queries and
adjacency use `ParticleInputCount`. Native dispatch, sorting, indexing, and
parameter encoding use explicit adapters to the existing GPU interfaces.
Empty requests still have zero active records. Mass-length rejection precedes
capacity rejection; position replacement checks the active count before taking
stored masses. Empty readback still visits the provider. Existing generic error
results remain unchanged. Hash bucket sizing retains unsigned arithmetic and
its build-dependent overflow behavior, rather than promising admission bounds.
