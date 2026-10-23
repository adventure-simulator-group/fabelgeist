# Radix-sort quantities

`RadixSort::record` and `run` accept separate `SortItemCount` and
`SortKeyWidth` values. A count is the number of native key/payload pairs; it
does not prove buffer size, usage, or membership in a particular allocation.
`SortScratch` retains that quantity for capacity and byte-layout admission.
Empty allocation requests retain the existing one-record sentinel. `ensure`
returns `fabelgeist_gpu::prelude::BufferCapacityChange::Unchanged` or
`BufferCapacityChange::Grown`.

Requested key widths clamp to one through 32 bits and schedule whole eight-bit
digits. `SortKeyWidth::pass_count` and `digits` own that schedule. Reduced widths
retain the caller's responsibility for the higher key bits. Each digit records
histogram, scan, and scatter in that order; odd pass counts copy keys and then
payloads back. The scan uses the shared parallel `PrefixSum`; each stage may
record more than one dispatch. Tile counts produce the canonical `WorkgroupGrid`
at the launch boundary, while histogram elements use `ScanElementCount`.

Zero and one item return before scratch and buffer admission. For larger counts,
scratch capacity is checked before logical byte lengths. Key/payload words,
stable ordering and shader word layouts retain their native representation.
Histogram layouts that exceed the native scan element count are rejected
before allocation or recording. Allocation and device limits remain the GPU
provider's responsibility. Scratch buffers are private so replacement cannot
invalidate retained capacity.

Consumers include BVH Morton ordering, vertex-normal corner gathering, cloth
self-collision buckets, and underlayer weld and triangle-grid entries. Their
populations and layouts remain distinct from launch quantities. The fixtures
pin original outputs, width clamping, scratch growth, rejection priority and the
histogram/scatter shaders. Existing prefix-sum checks cover the shared scan;
these fixtures do not establish device-limit acceptance.
