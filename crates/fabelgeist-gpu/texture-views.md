# Texture validation and format views

`Texture2d::view_with_format`, `Texture3d::view_with_format` and
`TextureCube::view_with_format` return
`TextureViewResult<Arc<wgpu::TextureView>>`. Their bespoke `TextureViewError`
preserves the method, original requested `TextureFormat` and native cause of a
rejected view. Generic callers use the standard `?` conversion into their
existing error boundary.

`TextureCube::face`, `face_view` and `face_view_with_format` return
`TextureViewResult<TextureView>`. A face view requires a native texture and
returns `Uninitialized` when it is absent. An observed failure records
`TextureViewOperation::TextureCubeFace` with its `CubeFace` and the original
requested format. The successful wrapper holds the selected native view, its
parent texture, face layer and effective format.

For whole-texture views, a request matching the stored format shares the cached
native view. Its Arc identity and sharing remain intact. If the cache is
absent, the method returns
`MissingView` with `Texture has no view`, including on a default texture or
when a caller clears the public view field. This replaces a panic. This branch
still precedes native texture admission, so a cached view can be returned even
if a caller clears the public native texture field.

A different-format request without a native texture returns `Uninitialized`
with `Texture is not initialized`. These two errors have no source.
`Observation` owns the operation, original requested format and the shared
`ValidationScopeError`. Both error enums derive their display and error
implementations from these payloads.

The scope error distinguishes a native rejection, failed polling, and failed
polling with a captured native rejection. The combined state owns both the
`wgpu::PollError` and `wgpu::Error`. Polling failures expose the polling error
as their primary source, with any captured rejection available in the combined
variant and its diagnostic. Native-only failures expose the native error.

```text
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
creation observes validation only when
`WgpuContext::validation_observation` selects
`ValidationObservation::BlockingScopes`. Borrowed and browser contexts select
`ValidationObservation::Uncaptured`, leaving native errors to the provider's
uncaptured-error handling. The same policy type is carried through shader,
pipeline and compute fast-path creation. Ownership and presentation are
independent of this observation choice.

Enabled observation attempts the native wait and then always pops its scope,
including after a polling failure. Its result selects `Native`, `Progress` or
`ProgressWithNative` from the actual polling and capture outcomes. No failure
returns the newly created view. This does not roll back earlier work or
establish the device's global health.

The three ComputePass texture-binding paths preserve these errors through
standard conversion. Their reflection and descriptor policies are unchanged.

`Texture2d::create`, `Texture3d::new` and `TextureCube::new` use the same
`ValidationScope` admission and completion. Their generic error boundary
propagates `ValidationScopeError` through standard conversion. Enabled
observation covers both native texture creation and its initial view; an
observation failure prevents publishing that newly constructed texture.

Cube-face selection observes validation only when supplied a context whose
policy is `BlockingScopes` on a native target. `face` supplies a context;
`face_view` does not. `face_view_with_format` accepts either choice. A
context-free face selection creates its native view without a validation-only
wait. Browser targets also omit synchronous observation. Array views, clearing
and readback remain separate operations. Readback completion and submission
backpressure retain their required waits, independently of validation policy.

Texture objects expose mutable native resources, caches and format metadata.
These error results do not certify that caller-mutated fields are coherent or
belong to the same device.
