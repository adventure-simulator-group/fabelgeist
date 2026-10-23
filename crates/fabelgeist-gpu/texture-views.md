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
after `clear_cached_view()`. This replaces a panic. This branch
still precedes native texture admission, so a cached view can be returned even
after `clear_native_texture()`.

A different-format request without a native texture returns `Uninitialized`
with `Texture is not initialized`. These absence errors have no source. The three
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
Storage binding requires an sRGB request to select its linear counterpart at
the descriptor boundary. The original request remains in a rejection. The
methods preserve descriptor inference, cube layers, labels and validation
filtering. Cached returns do not poll. Native contexts that own their device
read back the validation scope before returning; borrowed contexts skip that
blocking readback, as do browser builds. Those contexts report native failures
through the device's uncaptured-error handler rather than a synchronous
`TextureViewError`. Initial texture and cube-face view construction follow the
same context policy when given a context, retaining native causes on readback.

Texture allocation, cached view, logical format, extent and usage metadata are
controlled by the texture owner. Readonly accessors expose native ports and
metadata; removal operations cannot replace a resource or relabel its format.
All four allocation/cache availability combinations remain supported. An
alternate view requires the supplied context to use the creating device;
`WrongDevice` rejects a mismatch before native view creation. A cached-format
request retains its earlier cache-first behavior and does not use that context.

Compute bindings select a single handle with its effective format, dimension
and creating device. They validate those facts rather than the allocation's
format. A linear storage texture maps an alternate sRGB request back to its
linear view format. Whole-texture constructors exclude storage usage for logical
sRGB formats; generic cached views therefore retain one admitted format. Whole cube, face and array views retain Cube, D2 and D2Array
geometry. Face construction checks device identity when a context is supplied;
the context-free face API uses its retained native allocation. Array view
construction receives a context and retains native rejection in `CubeArray`.

`ComputePass` and the public `create_bind_groups` adapter share static binding
assembly. Both consume the immutable `ComputePipeline` and `PassParameters`,
preserving typed texture errors through standard conversion. Returned `ComputeBindGroup` values retain their
reflected indices, including sparse groups, and bind themselves to a native
pass. Logical buffer prefixes remain distinct from native allocation lengths.
`Buffer::binding_resource` rejects a zero-byte or oversized binding with
`BufferBindingError`; a zero logical extent remains valid as buffer metadata.

Static uniform assembly checks the padded native buffer extent against both
allocation and uniform-binding limits before packing. It allocates an unmapped
buffer and observes available native validation before uploading the bytes.
This avoids mapping a rejected allocation. The sRGB converter retains its
creating device, checks both cached texture devices and preserves available
native causes. Its fixed layout still accepts unfilterable float inputs for
`textureLoad`.

`ComputeShader` admits source, module and device together. `ShaderError` keeps
concrete Naga parse, validation and translation causes, source diagnostics and
available native shader failures. `ComputePipeline` keeps the immutable shader
and reflected interface, and caches complete outcomes by entry-point name.
`ComputePipelineError` preserves native rejection across clones and retries.
A rejected pipeline is never returned as a successful cache hit. Cache entries
also distinguish observed native validation from device-handled creation; a
later context requesting readback validates an earlier unchecked entry.
Preparation still reflects without compiling the static dispatch layout.
Compilation and recording check the actual creating device before using native
resources. `WorkgroupGrid` reaches the native dispatch boundary without raw
axis forwarding overloads.

The shared compute layout includes resources used by any compute entry point.
Reflection preserves sampler comparison, multisampling and storage access roles.
Float textures require filtering only when an entry point samples them; a
load-only float texture does not acquire that requirement. Provider features
and limits still determine native acceptance.

Named uniforms support one block per group: a flat struct or a direct scalar,
float vector or square float matrix. Scalars use f32 or the existing 32-bit
integer-word encoding, including signed inputs. Each reflected value retains its
expected shape. Static packing leaves missing or mismatched values at zero;
the cached path rejects them and retains its narrower matrix support. Supplied
values cannot overwrite another member by selecting a wider packing shape.
Multiple blocks in one group, nested or array uniform values, unsupported
scalar/vector/matrix representations and resource payloads absent from
`PassParameter` return typed pipeline admission errors. This includes resource
binding arrays and external textures. Unused globals require no bindings.
Used immediate data is also rejected explicitly: this interface supplies named
buffer and texture parameters, and records no native immediate-value commands.

Cached kernels keep per-batch uniform slots with checked byte offsets and
retain native pipeline/binding causes. Empty launches in a batch skip parameter
admission and counter increments. Standalone empty dispatches still admit
parameters.
Folded launches and callbacks after submission retain their existing ordering.
Borrowed contexts and browser builds still provide no synchronous native
validation verdict; typed errors cannot recover a cause the device never
reported through a read-back scope.

Public tests cover cache absence, branch priority, native rejection and retained
causes. Earlier software Vulkan observations covered view sharing, readback
bits and
compute-pass propagation with authored public reflection metadata. Those
historical observations do not validate the current source. They supply no
hardware-wide, browser runtime, performance or calibrated rendering verdict.
Current acceptance requires the hosted checks at the final PR revision.
