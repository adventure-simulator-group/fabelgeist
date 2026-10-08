# Radix-sort quantities

`RadixSort::record` and `run` accept separate `SortItemCount` and
`SortKeyWidth` values. A count is the number of native key/payload pairs; it
does not prove buffer size, usage, or membership in a particular allocation.
`SortScratch` retains that quantity for capacity and byte-layout admission.
Empty allocation requests retain the existing one-record sentinel. `ensure`
returns `ScratchGrowth::Unchanged` or `ScratchGrowth::Grown`.

Requested key widths clamp to one through 32 bits and schedule whole eight-bit
digits. `SortKeyWidth::pass_count` and `digits` own that schedule. Reduced widths
retain the caller's responsibility for the higher key bits. Each digit records
histogram, scan, and scatter in that order; odd pass counts copy keys and then
payloads back. Tile counts convert to native axes only at dispatch recording.

Zero and one item return before scratch and buffer admission. For larger counts,
scratch capacity is checked before logical byte lengths. Key/payload words,
stable ordering, source arithmetic, and shader text retain their native layout.
Generic allocation, compilation, dispatch, and copy errors remain separate work.
The fixtures pin original outputs, width clamping, scratch growth, rejection
priority, and native shaders; they do not establish device-limit acceptance.
