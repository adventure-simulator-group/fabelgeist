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

## Shader language selection

`ShaderLanguage` is the closed choice between WebGPU Shading Language (WGSL)
and OpenGL Shading Language (GLSL). `from_source_text` admits serialized shader
text and preserves the current lexical detection policy: any case-sensitive
`#version` occurrence selects GLSL, including occurrences in comments; other
text selects WGSL. Classification does not validate syntax.

The choice remains typed through Naga parser selection and the compute shader's
GLSL-to-WGSL conversion. Display retains the lowercase `wgsl` and `glsl` labels.
The former string-returning `detect_from_code` function is removed. Complete
source storage and structured shader errors are separate migration families;
native text at the source-construction and external parser boundaries remains.
