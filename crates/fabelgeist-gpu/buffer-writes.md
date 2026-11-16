# Checked buffer writes

`Buffer::write_at` admits one borrowed host upload at a byte address within the
buffer's logical extent. Its `BufferWriteResult` reports a `BufferWriteError`
instead of erasing the rejected request into a message. The bespoke error uses
the existing `BufferByteOffset` and `BufferByteLength` owners throughout.

First, the offset plus upload length must fit within the native u64 range.
`OffsetOverflow` retains the offset and length when that addition overflows.
Then the computed end must be at or before the public `Buffer::size` logical
length. `OutOfBounds` retains all three byte quantities. Neither rejection
queues a write. A successfully admitted upload reaches the native queue once.

```rust
use fabelgeist_gpu::prelude::{Buffer, BufferUpload, BufferWriteError};

# fn example(buffer: &Buffer, context: &fabelgeist_gpu::globals::WgpuContext) {
let words = [7u32, 8];
match buffer.write_at(context, 4u64.into(), BufferUpload::from_elements(&words)) {
    Ok(()) => {}
    Err(BufferWriteError::OffsetOverflow { at, bytes }) => {
        eprintln!("Cannot represent {bytes} bytes after {at}");
    }
    Err(BufferWriteError::OutOfBounds { at, bytes, length }) => {
        eprintln!("Cannot write {bytes} bytes at {at} within {length} bytes");
    }
}
# }
```

An empty upload can end exactly at the logical length, including logical zero
on an existing nonempty allocation. The byte owners admit all u64 values;
there is no global alignment or minimum policy. Allocation and its existing
nonempty policy remain separate from addressed-upload admission.

Logical extent admission does not prove native allocation capacity. The public
logical metadata can differ from the native allocation, and the queue still owns
alignment, copy-destination usage and allocation validation. Those errors
continue through the native SDK error mechanism even when `write_at` returns
`Ok(())`. Neither the error nor `BufferByteOffset::is_end_within` promises
native queue validity. Native integers are projected by the existing upload
adapter only when calling the queue; the logical end comparison stays with byte
owners.

Display text and the lack of a source error remain unchanged. Debug now exposes
variants and byte fields. Callers can match or downcast `BufferWriteError` when
using a generic outer error context; the previous message-only String or
borrowed-str downcasts no longer identify addressed-upload rejections.
