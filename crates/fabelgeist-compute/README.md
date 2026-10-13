# Compute kernel launches

`Kernel` admits its workgroup shape from the native shader declaration and
retains that shape as private metadata. `workgroup_shape()` returns its admitted
`WorkgroupShape`; `groups_for(InvocationCount)` produces a `WorkgroupGrid`
covering work along x with ceiling division. Counts become native integers at
the WebGPU command boundary.

`KernelBatch::dispatch` accepts an explicit grid, while `dispatch_items`
accepts an invocation count. Empty grids skip parameter admission and command
recording in a batch. Standalone `Kernel::run` admits parameters even for an
empty grid. Both APIs retain the same launch policy for nonempty grids.

`RecordedDispatchCount` increases after successful nonempty batch recording.
Copies, clears, empty grids, and rejected parameters leave it unchanged.
Observe GPU completion through submission, queue, or readback operations.

Kernel shape admission failures retain the shader entry-point label in
`WorkgroupDeclarationError` and the rejected dimensions in its
`WorkgroupShapeError` source. Format diagnostics for presentation and inspect
structured causes when handling a failure. Shader parsing, provider failures,
and general parameter errors continue to use their existing APIs.

Stereo depth recording separates depth conversion, temporal filtering, and
per-eye output. Each stage retains its parameter order, workgroup dimensions,
and position in the single submitted command sequence.
