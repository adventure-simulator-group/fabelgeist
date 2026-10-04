# GPU source and compilation

`ShaderSource` admits exact authored or assembled module text. It preserves
Unicode, whitespace, comments, and cache identity. Admission does not prove
syntax, validation, or device support. Keep it through CPU preparation,
reflection, shader creation, and kernel caching. Convert to `&str` only at the
Naga or wgpu representation adapter.

`ShaderEntryPoint` owns an exact entry-point label. `ShaderBindingName` owns a
resource label. Neither proves membership in a module, and neither can replace
source text. `binding_marker` preserves the generator's lexical `> name:`
query; its presence enum reports spelling, including comments and whitespace,
without claiming parsed reflection. Binding metadata retains
`ShaderBindingName`, distinct `BindGroupIndex` and
`BindingIndex` coordinates, and the shared `BufferByteOffset` and
`BufferByteLength` units. A resource label borrows its `PassParameterName`
lookup spelling, so dispatch does not allocate another label.

Language detection retains the exact `#version` marker policy. CPU preparation
keeps original source separately from emitted WGSL. `ShaderParseError`,
`ComputeShaderError`, and `ComputePipelineError` retain their compilation stage
and concrete provider causes. Format them through `Display` at presentation
boundaries; do not classify failures using rendered diagnostics.
`ComputePipeline::from_source` performs complete shader and pipeline
construction while retaining the shader preparation stage in its error.

`ValidationReadback` distinguishes contexts permitted to drain device work
from borrowed contexts that defer validation readback. Its scope operation
owns that decision. Borrowed device work must retain deferred behavior.

The shader source contract tests pin CPU conversion and complete diagnostics
against fixtures produced by the original implementation. Other tests cover
source admission, error classification, reflection layout, and type-family
separation. GPU and compute tests exercise actual owned and borrowed contexts.

`PassParameters` owns an ordered, private map of `PassParameterName` values.
Resources and uniform members share its existing flat namespace; admission
preserves exact spelling and replacement order. Names do not prove declaration
or availability. Keep nominal keys through resource lookup and pipeline-cache
construction, including secondary resources in map and stencil definitions.

`UniformNumber` admits numeric representations and rounds to binary32 only at
packing. `UniformUnsigned` retains unsigned words and signed admission casts.
These are shader ABI values; assign gameplay or mathematical roles before
using them as quantities elsewhere. `UniformBytes` owns little-endian packing
by reflected byte addresses. General packing preserves absent or unsupported
values and out-of-range regions. Cached packing requires each member and its
established scalar, vector or Mat4 subset. Preserve matrix column stride,
padding, signed zero, NaN, and cast behavior when extending either policy.

`ComputePassError`, `UniformPackingError`, and `TextureViewError` retain
resource roles, binding coordinates and concrete causes. Texture view selection
keeps the existing sRGB counterpart policy and admission order. A missing view
now returns a typed failure instead of panicking in `view_with_format`.
Uniform-arena slots retain shared byte lengths and addresses through a
private device-aligned stride and dynamic-offset representation. Convert to
wgpu words only when uploading bytes or recording the native pass.
`WorkgroupGrid` retains three-axis workgroup counts until the native command.
`WorkgroupShape` admits the shader's nonzero invocation dimensions and owns
coverage of an `InvocationCount` along x. Shapes, grids, and item counts cannot
substitute for one another. The original ceiling-division and empty-grid policy
are pinned against outputs from the original implementation, including maximum
word values. Device workgroup limits remain with pipeline admission.
`Buffer::new` retains `BufferByteLength` through allocation. `Buffer::length`
returns the private logical byte length, excluding native allocation padding.
`with_logical_length` selects a result within that allocation and returns
`BufferLengthError` for an oversized request. Empty logical results remain
valid, including absent mesh attributes; a physical allocation still must be
nonempty. `BufferCreationError::Empty` also survives initialized constructors.

Offset writes retain `BufferByteOffset` and classify overflow or a write past
the logical end with `BufferWriteError`, including nominal rejected quantities.
The previous admission order and diagnostic spelling are pinned against
original results. Zero-length writes at the end remain admitted. Alignment,
usage and native queue validation remain device contracts. Whole-buffer writes
retain their existing native validation policy and return unit, since that
operation has no recoverable host error.

`BufferUpload` admits exact `NoUninit` element bytes at the device-layout
constructor and borrows them through allocation or queue writes. It retains
nominal byte lengths and occupancy, without exposing its representation to
handwritten consumers. `with_empty_word` owns the established scalar-word
placeholder for empty armor inputs. Empty mesh attributes instead retain their
original backing allocation through `empty_result`, with no logical bytes.
Give values their geometry or gameplay roles before device-layout admission;
the upload payload does not establish an element's domain or shader layout.

`BufferLabel` owns exact allocation diagnostic spelling. `BufferDefinition`
keeps it and its usage selection private. `BufferUse` expresses independent
capabilities; storage includes both copy directions. An unspecified definition
uses the general device policy at allocation, while its first explicit choice
starts a fresh selection. `all` and `default` start with the general selection
already explicit. The original policy is pinned for all 512 combinations.
Mapping compatibility and device support remain native validation contracts.
Readback consults the native allocation's usage rather than duplicated mutable
metadata. Downstream element mathematics, layout-specific producers, texture
transfer and generic resource error layers remain migration work. Mesh
admission and transfer failures retain their roles in the
[owning mesh contract](../fabelgeist-mesh/README.md).
