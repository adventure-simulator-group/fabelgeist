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

## Shader source and language

`ShaderSource` carries complete shader text through parsing, compute-shader
creation and kernel caching. Admit borrowed text or an owned `String` with
`ShaderSource::from`; retained shader code and cache keys own their text.
Completed generators return `ShaderSource` and completed-source callbacks take
it directly. Static complete programs use `ShaderSource::borrowed`, which is
also usable in constants. Borrowed cache lookups do not allocate a copy of the
source.

```rust
use fabelgeist_gpu::prelude::{ComputeShader, ShaderSource, WgpuContext};

# fn example(context: &WgpuContext) -> anyhow::Result<()> {
let source = ShaderSource::from("@compute @workgroup_size(1) fn main() {}");
let shader = ComputeShader::new(context, source)?;
# Ok(())
# }
```

`source.language()` returns the closed choice between WebGPU Shading Language
(WGSL) and OpenGL Shading Language (GLSL). Any case-sensitive `#version`
occurrence selects GLSL, including comments; other text selects WGSL.
Classification does not validate syntax. Display retains the lowercase
`wgsl` and `glsl` labels.

Definitions, preambles and entry-point snippets remain text during assembly:
they are not complete programs. Admit the finished `String` once, moving it
into `ShaderSource`, then pass or borrow that source through application APIs.
Extract native text only for external parsers, device descriptors or explicit
text inspection. Parser diagnostics and structured shader errors remain
unchanged.
