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

`BufferUse` selects independent capabilities. An unspecified definition retains
the general allocation policy; selecting a capability makes that selection
explicit. Storage includes copy-source and copy-destination capability.

Empty uploads fail allocation. Armor consumers explicitly call
`with_empty_word` when an empty logical input still requires a scalar binding.
The generic buffer layer does not silently pad every input.

## Shader entry points

`ShaderEntryPoint` keeps an exact entry-point label distinct from shader source
and resource binding names. Convert authored labels or Naga parser output at
admission, then carry the bespoke type through kernel selection, compute
reflection and pipeline-cache lookup. Convert it to `&str` only at the native
WebGPU descriptor boundary.

Compute pipeline and kernel construction select the first declared compute
entry, even when a vertex entry precedes it or the compute entry is not named
`main`. `ReflectionData::compute_entry_point` records that selected label.
Selecting another cached pipeline requires an explicit `ShaderEntryPoint`.
Admission does not prove identifier syntax or membership in a shader module;
existing shader parsing and native validation still decide those questions.
The cache continues to hash the exact label without shader source in its key.
