# GPU buffers

`BufferUpload` borrows the native bytes of `NoUninit` host elements through
queue submission. `Buffer::from_upload`, `write`, and `write_at` consume that
upload. Convert host elements once, where the owning consumer admits them:

```rust
use fabelgeist_gpu::prelude::{Buffer, BufferDefinition, BufferUpload};

# fn example(context: &fabelgeist_gpu::globals::WgpuContext) -> anyhow::Result<()> {
let words = [1u32, 2, 3];
let buffer = Buffer::from_upload(
    context,
    BufferUpload::from_elements(&words),
    BufferDefinition::storage(),
)?;
# Ok(())
# }
```

Allocation lengths and write offsets use distinct byte types. Conversion to
native integers belongs at the WebGPU descriptor or queue boundary. Diagnostic
labels use `BufferLabel`; they are distinct from shader binding lookup keys.

`BufferElementStride` admits nonzero native storage-array byte spacing,
including padding. `BufferElementCount` derives complete records from a byte
extent and that stride, rejecting populations outside the native count range.
Neither quantity proves storage contents or source identity. Array elements,
byte extents and shader invocations remain distinct quantities.

`InvocationCount` counts shader invocations; `WorkgroupCount` counts x-axis
workgroups. `WorkgroupShape` derives group demand from the native declaration,
and `WorkgroupGrid` carries the launch axes. One-record-per-group algorithms
convert explicitly between the stored-record and workgroup quantities.

`BufferUse` selects independent capabilities. An unspecified definition retains
the general allocation policy; selecting a capability makes that selection
explicit. Storage includes copy-source and copy-destination capability.

Empty uploads fail allocation. Armor consumers explicitly call
`with_empty_word` when an empty logical input still requires a scalar binding.
The generic buffer layer does not silently pad every input.

The [whole-texture format-view contract](texture-views.md) describes view
selection errors, native causes and cached-view sharing.
