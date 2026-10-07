# Physical buffer creation

`Buffer::new` and `Buffer::from_upload` return `BufferCreationResult`, whose
bespoke `BufferCreationError::Empty` means the requested physical allocation has
zero bytes. The exact diagnostic is `Buffer size must be greater than 0`; it has
no source error. This check happens before native graphics allocation or upload.
Use `?` to preserve the concrete cause when adding an outer application context.

`BufferByteLength` still admits zero. Logical mesh metadata, empty host uploads
and native byte quantities share that existing owner. Its `is_empty` predicate
expresses the zero test without extracting a byte quantity into domain logic.
Creation imposes the existing physical allocation policy, not a global minimum
on every byte length.

`Buffer::from_upload` allocates the exact upload length and adds
copy-destination capability before sending the borrowed upload once at
`BufferByteOffset::START`. The existing typed `BufferUpload::write_to` projects
its address and byte slice only at `Queue::write_buffer`. Allocation definitions
project byte lengths and borrowed labels at the native descriptor. No extra
submission, polling or native validation is added.

Empty uploads, including nonempty slices of zero-sized host elements, reject.
Consumers may explicitly use `BufferUpload::with_empty_word` when an empty input
still needs a scalar binding. Empty meshes retain their own physical padding
and logical zero metadata. Creation does not silently pad arbitrary inputs.

Native graphics validation remains provider-owned. A nonzero constructor can
return success while a native error scope reports invalid capabilities or upload
alignment. These are not `Empty` failures. Device allocation limits, native
panic/error-buffer behavior and queue timing retain their existing policies.
Public `Buffer::write` and `write_at` have separate result contracts.

Creation-only helpers carry `BufferCreationResult`. Operations that also read
or validate data convert the concrete error into their existing outer result
with standard error conversion. This preserves a downcastable `Empty` cause
without wrapping arbitrary failures in a constructor error.

Public `Buffer::buffer` sharing and clone equality are observable through its
native `Arc`. Payload readback also requires native copy-source capability;
index-only wireframe outputs do not promise host readback. Software Vulkan
observations establish bounded native behavior, not a verdict for every device,
browser runtime, allocation size or renderer.
