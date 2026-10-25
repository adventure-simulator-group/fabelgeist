# Whole-texture format views

`Texture2d::view_with_format`, `Texture3d::view_with_format` and
`TextureCube::view_with_format` return
`TextureViewResult<Arc<wgpu::TextureView>>`. Their bespoke `TextureViewError`
preserves the method, original requested `TextureFormat` and native cause of a
rejected view. Generic callers use the standard `?` conversion into their
existing error boundary.

A request matching the stored format shares the cached native view. Its Arc
identity and sharing remain intact. If the cache is absent, the method returns
`MissingView` with `Texture has no view`, including on a default texture or
after `clear_cached_view()`. This replaces a panic. This branch still precedes
native texture admission, so a cached view can be returned even after
`clear_native_texture()`.

A different-format request without a native texture returns `Uninitialized` with
`Texture is not initialized`. These absence errors have no source. The three
native rejection variants own a real `wgpu::Error` and expose it through
`std::error::Error::source`. They retain the existing diagnostic prefixes and
original requested format. Their Debug representation and cause chain are
structured; the old generic String or str message identity is removed.

```rust
use fabelgeist_gpu::{globals::WgpuContext, prelude::*};
use std::sync::Arc;

fn select(
    context: &WgpuContext,
    texture: &Texture2d,
) -> TextureViewResult<Arc<wgpu::TextureView>> {
    texture.view_with_format(context, TextureFormat::Rgba8UnormSrgb)
}
```

The WebGPU device, native view descriptor and returned Arc are SDK boundaries.
Storage binding requires an sRGB request to select its linear counterpart at the
descriptor boundary. The original request remains in a rejection. The methods
preserve descriptor inference, cube layers, labels and validation filtering.
Cached returns do not poll. Native contexts that own their device read back the
validation scope before returning; borrowed contexts skip that blocking
readback, as do browser builds. Those contexts report native failures through
the device's uncaptured-error handler rather than a synchronous
`TextureViewError`. Initial texture and cube-face view construction follow the
same context policy when given a context, retaining native causes on readback.

Native observation attempts the existing wait once and always reads the scope,
even when polling fails. `ValidationProgressError` retains the actual
`wgpu::PollError` and any validation error captured by that scope. View failures
carry it in `TextureViewError::Progress`, together with the original requested
format and view operation. Ordinary native rejections retain their existing
variants, messages and direct SDK causes. The wait requests no explicit timeout;
a backend can still fail to make progress. These errors do not establish global
device health.

Texture allocation, cached view, logical format, extent and usage metadata are
controlled by the texture owner. Readonly accessors expose native ports and
metadata; removal operations cannot replace a resource or relabel its format.
All four allocation/cache availability combinations remain supported. An
alternate view requires the supplied context to use the creating device;
`WrongDevice` rejects a mismatch before native view creation. A cached-format
request retains its earlier cache-first behavior and does not use that context.

Compute bindings select a single handle with its effective format, dimension and
creating device. They validate those facts rather than the allocation's format.
A linear storage texture maps an alternate sRGB request back to its linear view
format. Whole-texture constructors exclude storage usage for logical sRGB
formats; generic cached views therefore retain one admitted format. Whole cube,
face and array views retain Cube, D2 and D2Array geometry. Face construction
checks device identity when a context is supplied; the context-free face API
uses its retained native allocation. Array view construction receives a context
and retains native rejection in `CubeArray`.

`ComputePass` and the public `create_bind_groups` adapter share static binding
assembly. Both consume the immutable `ComputePipeline` and `PassParameters`,
preserving typed texture errors through standard conversion. Returned
`ComputeBindGroup` values retain their reflected indices, including sparse
groups, and bind themselves to a native pass. Logical buffer prefixes remain
distinct from native allocation lengths. `Buffer::binding_resource` rejects a
zero-byte or oversized binding with `BufferBindingError`; a zero logical extent
remains valid as buffer metadata.

Static uniform assembly checks the padded native buffer extent against both
allocation and uniform-binding limits before packing. It allocates an unmapped
buffer and observes available native validation before uploading the bytes. This
avoids mapping a rejected allocation. If an operation and its later observation
both fail, `ComputePassError::OperationAndObservation` keeps the operation as the
primary cause and exposes the additional observation error. The sRGB converter
retains its creating device, checks both cached texture devices and preserves
available native causes. Its fixed layout still accepts unfilterable float inputs for
`textureLoad`.

`ComputeShader` admits source, module and device together. `ShaderError` keeps
concrete Naga parse, validation and translation causes, source diagnostics and
available native shader failures. `ComputePipeline` keeps the immutable shader
and reflected interface, and caches complete outcomes by entry-point name.
`ComputePipelineError` preserves native rejection across clones and retries. A
rejected pipeline is never returned as a successful cache hit. Cache entries
also distinguish observed native validation from device-handled creation; a
later context requesting readback validates an earlier unchecked entry.
A progress failure is transient: it is not cached and removes an earlier
unchecked entry for the same entry point, so a retry observes creation again.
Failed observation does not complete compilation. Preparation still reflects
without compiling the static dispatch layout.
Compilation and recording check the actual creating device before using native
resources. `WorkgroupGrid` reaches the native dispatch boundary without raw axis
forwarding overloads.

The shared compute layout includes resources used by any compute entry point.
Reflection preserves sampler comparison, multisampling and storage access roles.
Float textures require filtering only when an entry point samples them; a
load-only float texture does not acquire that requirement. Provider features and
limits still determine native acceptance.

Named uniform destinations are the immediate members of a reflected struct.
Their native byte offsets and spans use `BufferByteOffset` and
`BufferByteLength` through reflection, static packing and the cached path. Both
packers choose the provided `PassParameter` variant, preserving its native byte
encoding and the existing whole-block bounds. They do not require that variant
to match the member's declared shader type or restrict writes to the member's
span. Array and nested members therefore retain the same provided-value protocol
as other struct members. Signed and unsigned inputs keep their existing 32-bit
word representation.

Static packing starts with zero-filled bytes and skips missing or unsupported
provided values. The cached path rejects them and retains its narrower matrix
support. Direct nonstruct blocks retain their native extent but have no named
destinations, so they stay zero initialized; an extra parameter keyed by the
global name does not change them. Matrix packing preserves the native component
order and the existing reflected-span rule for Mat3. A component window that
would run beyond the block now returns a typed packing error instead of
panicking. Host address narrowing retains its concrete conversion cause.

Multiple uniform blocks in one group remain unsupported by this interface.
Resource binding arrays and external textures have no `PassParameter` payload
and return typed pipeline admission errors. These binding arrays are distinct
from array values inside one uniform block. Unused globals require no bindings.
Used immediate data is also rejected explicitly: this interface records no
native immediate-value commands.

Cached kernels keep per-batch uniform slots with checked byte offsets and retain
native pipeline/binding causes. Empty launches in a batch skip parameter
admission and counter increments. Standalone empty dispatches still admit
parameters. Folded launches retain their existing ordering. Recording a dispatch
or copy does not establish native encoding acceptance: wgpu can defer validation
until encoder finalization. A failed observation does not undo commands already
recorded in a caller's encoder. Cached preparation can also consume a uniform
arena slot before a later observation fails. Conversion submission can report
an observation failure after work has been submitted. None of these failures
promises rollback. `WgpuContext::submit` observes Validation errors from
finalization before queue submission, then observes submission separately under
the context's native validation policy. It returns the actual SDK cause without
waiting for GPU completion. The encoder must be created by the context's device,
and its queue must belong to that same logical `wgpu::Device`; sharing a backend
or adapter is insufficient. Native builds explicitly enable wgpu's `std` feature
so its scope tracking distinguishes threads.

`KernelBatch::submit` and standalone dispatch preserve these causes through
`ComputePassError::Native`. Batch application callbacks run in registration
order only after the submission helper returns `Ok`; a discarded or rejected
batch drops them unrun. These callbacks can initiate mapping, but do not report
GPU completion. A pending readback obtained before submission must be discarded
when submission fails; callback removal does not supply a mapping outcome.
Separate readback or completion operations remain necessary to obtain results.
This boundary captures the Validation filter, not every possible device error.
Borrowed contexts and browser builds still provide no synchronous
native validation verdict; typed errors cannot recover a cause the device never
reported through a read-back scope.

Public tests cover cache absence, branch priority, native rejection and retained
causes. Earlier software Vulkan observations covered view sharing, readback bits
and compute-pass propagation with authored public reflection metadata. Those
historical observations do not validate the current source. They supply no
hardware-wide, browser runtime, performance or calibrated rendering verdict.
Current acceptance requires the hosted checks at the final PR revision.
