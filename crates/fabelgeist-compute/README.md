# Kernel compilation and caching

`Kernel::new` accepts `ShaderSource` and returns `KernelBuildError`. A kernel
retains its nominal entry-point label through native pipeline creation.
Missing compute entries, rejected workgroup declarations, parser failures,
shader preparation failures, and device pipeline failures stay classifiable.
Provider errors retain their original sources.

`KernelCache` keys exact `ShaderSource` values. Whitespace and comments
distinguish modules; no normalization or pre-hashing changes that identity.
Compilation stays outside the cache write lock. Concurrent first uses may
compile separately, but the first inserted pipeline wins and every successful
caller shares it. `CompiledKernelCount` counts distinct sources rather than
dispatches or workgroups.

`KernelCacheError` distinguishes read/write poisoning from compilation failure.
Poisoned cache access returns a typed error instead of panicking. Compilation
failures retain source and `KernelBuildError`; armor and creator consumers
preserve that chain through their generation errors.

`KernelDispatchError` distinguishes general-pass failures, cached buffer
admission, and uniform packing. Armor and creator dispatch consumers preserve
this cause instead of storing its rendered message. The cached path uses the
GPU owner's nominal binding metadata and `UniformBytes` packing policy while
retaining per-dispatch bind groups and batch-owned uniform storage.

`Kernel` retains an admitted `WorkgroupShape` from its source, with nonzero
invocation dimensions. `InvocationCount` covers x items through checked-shape
ceiling division, producing a distinct `WorkgroupGrid`. General and cached
recording retain that grid until the native command. Kernel item coverage still
dispatches one group along y and z; it does not flatten a multidimensional
declaration. Device limits remain the pipeline provider's responsibility.

`KernelDispatchPath` reports the selected recording path. A batch skips any grid
with a zero axis before parameter admission; standalone general recording keeps
its existing admission behavior for empty grids. `RecordedDispatchCount` counts
successful recorded dispatches, including neither skipped work nor buffer
copies. It is distinct from item, workgroup, and compilation quantities.

`KernelBatchLabel` borrows exact diagnostic spelling through encoder creation.
Batch copies retain `BufferByteLength` and classify out-of-bounds requests with
`BufferCopyError`, including both logical buffer extents. Generated source
fragments, upstream algorithm quantities, and other execution errors remain
interface migration work. Their existing behavior does not exempt them from
the census. There are no primitive forwarding overloads or compatibility paths.

`PassParameters::overlay` combines an ordered parameter bundle without changing
the position of replaced names. `ShaderParameterTypes` admits parsed declaration
names before secondary-resource lookup and preserves the first declaration's
type. Map and stencil caches retain nominal parameter keys.

`UniformStride`, `UniformSlot`, and `UniformDynamicOffset` keep batch-owned
uniform extents, addresses, and device alignment through allocation and upload.
The native descriptor and command calls encode those quantities at their SDK
boundaries. The arena's chunk size and rollover behavior remain fixed, and bind
groups are still created for each dispatch to retain dependency barriers.

`SurfaceAttributes` shares position and normal extraction for marching cubes
and dual contouring. It retains nominal byte extents and dispatch grids through
allocation and recording, preserving interleaved vertex words and labels.
Allocation and dispatch failures retain their concrete causes.

## Radix sorting

`SortItemCount` counts key/payload pairs, while `SortKeyWidth` selects the
low-bit digit schedule. Width construction retains the existing clamp to one
through 32 bits. Counts, tiles, pass counts, and digit shifts retain distinct
roles through scratch layout, parameter binding, dispatch, and final copies.
The key and payload buffers keep their native four-byte words.

Zero and one item return before scratch or buffer admission. Scratch allocation
retains a one-item sentinel; larger requests check scratch capacity before
buffer byte lengths. `ScratchGrowth` reports whether replacement succeeded.
Odd pass counts copy both sorted buffers back to the caller's buffers. Higher
key bits remain the caller's responsibility when selecting a reduced width.

`SortScratchError` retains the failed allocation's buffer role, capacity, byte
extent, and concrete provider cause. `SortBuildError` identifies the compilation
stage. `SortError` distinguishes capacity and byte admission, digit dispatch,
and final buffer copies while retaining concrete causes. BVH, vertex-normal,
self-collision, and creator sorting calls use these nominal sorting quantities.
Their surrounding primitive geometry interfaces and generic errors remain
separate migration work.
