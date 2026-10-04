# Mesh admission and draw topology

`MeshData` validates attribute counts, finiteness, skin pairing, index bounds,
and primitive grouping in their established order. Empty normals and texture
coordinates mean absent attributes. Skin weights must be finite and nonnegative;
admission does not require normalized sums or prove joint membership in a rig.
Negative zero remains admitted.

`MeshValidationError` retains the rejected attribute, nominal lengths, missing
skin role, weight reason, or draw address. `MeshTriangleError` keeps validation
separate from unsupported topology. Triangle strip expansion keeps the original
ordinal's winding parity when it skips a degenerate triple. Errors retain their
original diagnostic spelling through `Display`; callers use variants rather than
classifying that prose.

`DrawVertexCount` owns a mesh's 32-bit draw cardinality, including zero.
`DrawVertexIndex` owns an address in that draw space. Admission of an address
does not prove membership in a particular mesh. Counts, indexes, and
`NeighborCapacity` cannot substitute for each other. These draw addresses are
distinct from simulation particle identities and authored model topology
cardinalities. `MeshAttributeLength` retains host collection lengths, including
values too large for draw admission.

Topology decoding assigns draw identities before edge construction. Canonical
wire edges exclude loops and endpoints outside the mesh. Target-neighbor lines
retain the supplied target while filtering the other endpoint, preserving the
established target policy. Fixed-stride neighborhood records own their count
word, capacity clipping, and zero padding. Line serialization owns the original
two-zero-word empty sentinel. Hash sets keep their original traversal policy;
capacity truncation does not introduce an ordering promise.

`MeshUploadError`, `MeshReadbackError`, `MeshTopologyTransferError`, and
`MeshDisplacementError` retain operation or attribute roles and concrete
allocation, readback, pipeline construction, or dispatch causes. Readback keeps
the original order and logical-length truncation. Empty mesh attributes retain
16 physical backing bytes and zero logical bytes. An empty neighborhood
allocation still fails; line outputs use their own sentinel policy instead.

`PlaneGeometry`, `BoxDimensions`, and `SphereGeometry` own construction through
GPU upload. `PlaneSubdivisions` distinguishes cell counts from coordinates;
`SphereRings` and `SphereSectors` distinguish latitude from longitude.
`SphereRadius` admits signed mesh-space radii and retains the original binary32
cast. Cell admission preserves the existing minimum, truncation, and saturation
policies, including NaN and infinity. Admission does not guarantee that host
allocation arithmetic or draw counts fit. The original defaults, vertex order,
winding, index order, and box basis arithmetic remain unchanged. Executable
fixtures from the original implementation pin validation, topology, and actual
GPU shape bytes.

`MeshDisplacement` owns a reusable pipeline and borrows the context that created
it. Retain the operation to reuse that pipeline. Construction returns a
`MeshDisplacementError::Pipeline` with its concrete cause instead of panicking;
it no longer uses a process-wide cache across devices. Source mesh and texture
resources still need to belong to that context, as required by the provider.
`DisplacementProjection` keeps projection and inverse together, preserving the
default inverse for singular matrices. `DisplacementScale` retains signed and
nonfinite admission and rounds to binary32 at uniform binding. Dispatch retains
its original position-byte narrowing before vertex division; draw metadata does
not replace that ABI extent. Fixtures pin original displaced position bits.

The attribute fields, triangle-array return representation, and general math
interfaces remain migration work. Device limits and native alignment validation
remain provider contracts. Keep these remaining interfaces visible in the
census; their presence does not justify broad mesh or math exceptions.
