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
when a caller clears the public view field. This replaces a panic. This branch
still precedes native texture admission, so a cached view can be returned even
if a caller clears the public native texture field.

A different-format request without a native texture returns `Uninitialized`
with `Texture is not initialized`. These two errors have no source. The three
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
same context policy and retain native causes when a scope is read back.

The three ComputePass texture-binding paths preserve these errors through
standard conversion. Their reflection and descriptor policies are unchanged.
Cube-face views, array views, texture creation, clearing and readback remain
separate operations. Further subresource work should compose this error owner
where applicable rather than introduce an equivalent type.

Public tests cover cache absence, branch priority, native rejection and retained
causes. Earlier software Vulkan observations covered view sharing, readback
bits and
compute-pass propagation with authored public reflection metadata. Those
historical observations do not validate the current source. They supply no
hardware-wide, browser runtime, performance or calibrated rendering verdict.
Current acceptance requires the hosted checks at the final PR revision.
