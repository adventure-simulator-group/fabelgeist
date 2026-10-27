Whole-texture format-view requests currently erase native validation causes into
text, panic when a matching-format cached view is absent, and ignore failed
device polling. `Texture2d`, `Texture3d` and `TextureCube::view_with_format` now
return a shared `TextureViewResult` preserving the requested format and native
cause. Missing resources return typed errors, and failed observation returns a
typed progress error before caching a new view.

The shared `ValidationScope` keeps its device and guard together, attempts
polling before always popping the scope, and retains both polling and captured
validation errors. `ValidationObservation::{BlockingScopes, Uncaptured}` carries
the same policy through the context, pipeline API, shaders, texture creation and
compute fast path. Existing-device and browser contexts select uncaptured
errors. Presentation and device provisioning remain independent choices.

Cached Arc identity, cache-first precedence, storage sRGB projection, native
descriptors and the three ComputePass error recipients preserve their behavior.
The workflow retains its existing Linux, Windows, fixture-exporter and
positive-execution gates. No tests, test cases, test helpers, assertions or doctests are
added; the existing borrowed-context test's two predicates now select the enum.

[Hosted run 40397904316](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/40397904316)
is validating commit `dd1e69eca683a5aaee26ec93b7ad3cddf6de8720`.
Final validation and the relevant hosted baseline reduction remain pending.

## Implementation context

Independent Code, Rule and Comment source reviews cover the original change and
the policy and derive corrections, including connected callers and native
macro and SDK implementations. `ValidationScopeError` derives `Display` and
`Error`; `TextureViewError` derives `Display`. Its source projection remains
explicit so downcasts reach each concrete error behind its box. The progress
box has a different concrete type from the three native-error boxes.

Existing tests do not directly exercise the new view-error or polling-failure
branches; their behavior is supported by source traces. Public texture, cache, metadata and device coherence,
constructor and subresource validation, remaining migrations and the complete
audit stay open. This is an independent increment
toward #826, connected to #780 and #726.
