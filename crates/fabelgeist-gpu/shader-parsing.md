# Shader frontend parsing

`shader::parse_naga` parses WebGPU Shading Language (WGSL) or OpenGL Shading
Language (GLSL) into a native Naga module. Its bespoke `ShaderParseError`
classifies those two frontend rejections; `ShaderParseResult` is its concrete
result. Source containing `#version`, including comments, selects GLSL. Other
source selects WGSL. Native shader stages and parser admission remain unchanged.

The plain diagnostic retains the existing prefix and native formatting. WGSL
source excerpts are rendered once while source is borrowed; the owned diagnostic
and native cause outlive it. GLSL retains native error-list Debug formatting.
`Error::source` exposes the actual Naga cause. Debug and expanded cause-chain
formatting therefore differ from the former message-only error.

ComputeShader and ComputePipeline preserve their complete presentation prefixes
through standard anyhow context while retaining the frontend root and native
cause. Other compute builders propagate it through standard error conversion.
An outer context can have its own downcast independently of the owning root.

Frontend success does not promise validator or device acceptance. Validation,
reflection, descriptors, pipeline baking and native SDK errors retain their
existing policies. ComputeShader now uses its first native validator result;
this removes redundant evaluation and the panic if a second evaluation were
inconsistent. ComputePipeline carries its locally admitted reflection Arc to
native baking, removing the constructed-Some extraction panic. Neither cleanup
introduces a fallback, another rejection or a skipped bake.

```compile_fail
use fabelgeist_gpu::data::gpu::shader::parse_naga;

let _: anyhow::Result<wgpu::naga::Module> =
    parse_naga("", wgpu::naga::ShaderStage::Compute);
```

The concrete result requires deliberate conversion at a mixed-error boundary;
it cannot silently become the old generic parser result. Native module fields,
source text, ShaderStage and formatter output are representation ports. No
additional source, language, entry-point or descriptor authority is introduced.

Public regression fixtures cover both frontends and cause retention without a
GPU. Software GPU observations can additionally check compilation, reflection
and bounded readback, but establish neither browser nor calibrated-renderer
behavior or performance.
