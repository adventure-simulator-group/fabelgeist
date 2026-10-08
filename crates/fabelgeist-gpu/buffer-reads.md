# Buffer readback

`Buffer::read` admits a logical byte extent before native GPU work and returns
`BufferReadResult<Vec<T>>`. Its bespoke `BufferReadError` distinguishes a host
length conversion, zero-sized element, partial element, truncated mapped view,
native mapping failure and canceled callback channel.

Logical extents and diagnostic byte fields use the existing `BufferByteLength`.
The private host destination stores that extent and checks host conversion
before checking element size and divisibility. Conversion to native usize is
limited to host allocation, pointer copy and vector length. Mapped-view lengths
enter from the SDK's byte slice. There is no new allocation or alignment policy.

Plain diagnostics are unchanged. Debug and concrete downcasts identify the
owning error; `Error::source` retains `TryFromIntError`, `BufferAsyncError` or
native channel cancellation. Mixed-resource and mesh APIs still return anyhow
results: standard conversion preserves the concrete buffer root. Outer context
strings remain separate from that root. Armor retains its Display adapter.

Cached MAP_READ usage still selects direct mapping; other buffers use the same
staging copy and submission. The mapped view is dropped before unmapping either
buffer, including a truncated host copy. Mapping rejection still returns before
that cleanup; polling errors and callback lifetime policy are unchanged.
Direct mapping does not submit pending queue writes. A caller that requires an
upload completed before reading uses the existing context completion API.

Zero logical bytes remain distinct from zero-sized elements: a directly mapped
nonempty allocation can return an empty result, whereas a zero staging slice can
panic in the native SDK. Mesh's empty shortcut remains its consumer policy.
Native SDK alignment, usage and physical extent rejection retain their existing
behavior, including possible zero data from scoped invalid copies. This error
contract adds no stricter wire, allocation, copy or provider validation.

Native software-GPU checks establish API behavior and causes, not calibrated
rendering or performance. Host conversion failure requires a narrower host;
browser callback behavior and public cancellation need separate runtime proof.
