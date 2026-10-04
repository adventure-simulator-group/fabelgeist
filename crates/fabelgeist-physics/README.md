# Body collision resources

`ColliderCount` retains the full host cardinality of an analytic shape list.
`ColliderCapacity` describes allocated native collider records. Keep these
roles distinct through replacement, growth, count diagnostics and parameter
encoding. Initial allocation retains 16 records; growth retains the original
host power-of-two rounding followed by unsigned device narrowing. Smaller and
empty replacements retain the existing allocation.

Native count narrowing remains the existing boundary contract. The owning
capacity comparison and shader encoding preserve it; constructing a host count
does not prove that the list fits the device word. Do not introduce an overflow
or allocation admission change while lifting these types without separately
establishing the intended behavior. Shape packing remains sixteen native words
per collider, with the existing kind bits and padding.

`CollisionBuildError` retains analytic or mesh kernel preparation and allocation
capacity with concrete cache or buffer causes. `ColliderUpdateError` retains
full held and provided counts, or a growth allocation failure. A rejected count
replacement fails before uploading or changing the held list. Growth keeps its
existing capacity assignment before allocation, including its failure state.

`CollisionRecordError` distinguishes analytic and mesh dispatches from the
previous-position copy used by push-out. It retains the concrete binding or
copy-bounds failure. Inspect variants and cause chains to classify these
failures; display text keeps the lower provider's established message.

Ordinary collision skips disabled state before empty particle count. Push-out
retains its separate contract: it skips empty particles, copies their current
positions to the previous state, and resolves the mesh even when ordinary
collision is disabled. Neither operation changes tactical authority or storage.

Ten independently executed original scenes pin thirty GPU states, including
every analytic shape, mesh and combined collision, collider growth, updated
surface parameters, disabled and empty paths, and push-out. The fixture retains
5,328 native bytes from positions, previous positions and velocities, including
inverse masses and record padding. Tests compare complete typed records at this
serialization boundary.

Shape and mesh geometry, radius, friction, enable state, push-out pass counts,
and the handwritten `SubstepHook`/`HookChain` interfaces remain explicit
migration debt. Their generic hook adapter retains `CollisionRecordError` as a
concrete cause. This internal trait is not an external API exception.
