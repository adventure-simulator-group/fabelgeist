# Compute-batch diagnostic labels

`KernelBatchLabel` identifies a batch's native command encoder in GPU
validation diagnostics. It is a borrowed bespoke type with private contents.
It admits every native string exactly, including empty strings, Unicode,
whitespace, control characters and NUL. Admission neither allocates nor changes
spelling, and has no error policy.

Construct labels explicitly at authored literals or native string inputs:

```rust
use fabelgeist_compute::{KernelBatch, KernelBatchLabel};
use fabelgeist_gpu::prelude::WgpuContext;

fn batch(context: &WgpuContext) -> KernelBatch<'_> {
    let label = String::from("simulation step");
    KernelBatch::labelled(context, KernelBatchLabel::from(label.as_str()))
}
```

The label only needs to live through encoder creation. The returned batch
borrows its context independently; the local String above is dropped before
the batch is used. `KernelBatch::new` supplies the exact default spelling
`KernelBatch`. `ArmorGpu::batch` and the private metal-bake helper accept and
forward the same admitted label.

Only the native `wgpu::CommandEncoderDescriptor` converts the label back to a
borrowed string. The SDK copies that spelling during creation. Deferred native
validation diagnostics retain nonempty labels; an empty label is admitted but
has no label clause in those diagnostics. SDK timing, validation and panic
policies are unchanged. Labels select neither a kernel nor a cache entry and
have no effect on work ordering, dispatch counts or submission.

Buffer allocation labels have their own `BufferLabel` owner. Per-pass labels,
debug markers, native submission indices and dispatch counts have different
responsibilities and are not encoded by this type. There is no raw-string
forwarding overload or conversion through another label owner.
