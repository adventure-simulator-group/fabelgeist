# Batch buffer copies

`KernelBatch::copy_buffer` records a copy between two buffers at their start,
ordered against the surrounding dispatches in the same batch. Its requested
length uses the existing GPU `BufferByteLength` bespoke type. Admit a native
byte count with `BufferByteLength::from` at a literal or representation port.
The length type accepts every `u64`, including zero and `u64::MAX`.

The batch compares that length against both public `Buffer.size` values.
A request exceeding either logical extent returns `BufferCopyError`, with typed
`bytes`, `source_length` and `destination_length` fields. The error's display
text retains the requested and available byte lengths; it has no source error.
An `anyhow::Error` propagated by a consumer retains this error for downcasting.
Its structured debug output identifies the same fields. A rejected copy records
no command and leaves already recorded work and dispatch count unchanged.

Zero, partial and full copies that fit both extents are admitted. This admission
uses the logical metadata, even when it differs from the native allocation.
Alignment, buffer usage, overlap and allocation validation remain with the GPU
SDK; an admitted copy can still fail that later validation. Copies do not count
as dispatches. This method adds no padding, saturation or extra capacity check.

The native SDK call needs a `u64` length, so projection occurs directly there.
Its zero offsets mean the beginning of each buffer. Radix sorting admits the
existing element-count times four calculation because keys and payloads are
native `u32` arrays. Contact recovery admits active-particle-count times sixteen
because a position record contains four native `f32` values. Both consumers
retain the typed length through checks and copies; contact recovery preserves
the active prefix and leaves capacity tails untouched.

The raw `u64` batch argument is removed. Buffer allocation, dispatch dimensions,
particle counts, sort digits and unrelated generic consumer errors retain their
own responsibilities. The copy error describes logical extent rejection only.
