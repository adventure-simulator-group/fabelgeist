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
structured.

`Progress` retains the operation, original requested format and a
`ValidationProgressError`. That cause owns the actual `wgpu::PollError` and any
native validation error captured by the same scope. Its source prefers the
captured native error when present, otherwise the polling error; both remain
available as fields.

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
method preserves descriptor inference, cube layers, labels and validation
filtering. Cached returns do not poll. On native targets, alternate-format view
creation observes validation only when `WgpuContext::blocking_validation` is
enabled. Borrowed contexts disable that policy and leave native errors to the
provider's uncaptured-error handling. Browser targets do not synchronously wait
for a native validation scope.

Enabled observation attempts the native wait and then always pops its scope,
including after a polling failure. A captured rejection returns its native
error; a failed wait returns `Progress` and retains any captured rejection.
Neither path returns the newly created view after failed observation. This does
not roll back earlier work or establish the device's global health.

The three ComputePass texture-binding paths preserve these errors through
standard conversion. Their reflection and descriptor policies are unchanged.
Cube-face views, array views, texture creation, clearing and readback remain
separate operations. Further subresource work should compose this error owner
where applicable rather than introduce an equivalent type.

Texture objects expose mutable native resources, caches and format metadata.
These error results do not certify that caller-mutated fields are coherent or
belong to the same device. Constructors and subresource methods have their own
validation paths and do not yet consistently honor the context's blocking
policy.
