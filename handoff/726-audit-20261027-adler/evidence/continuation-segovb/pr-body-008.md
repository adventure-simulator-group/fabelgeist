Whole-texture format-view requests currently erase native validation causes into
text, panic when a matching-format cached view is absent, and ignore failed
device polling. `Texture2d`, `Texture3d` and `TextureCube::view_with_format` now
return a shared `TextureViewResult` carrying the requested format, operation and
validation failure. Missing resources return typed errors; failed observation
returns an error before returning the newly created view.

`TextureViewError::Observation` directly owns `ValidationScopeError`. The scope
error distinguishes native rejection, failed polling, and failed polling with a
captured native rejection. The combined state owns both causes. Polling is the
primary source when it fails; the captured native rejection remains available
in its named field and diagnostic. Both enums derive `Display` and `Error`.

The shared `ValidationScope` keeps its device and guard together and always pops
the scope after attempting polling. `ValidationObservation::{BlockingScopes,
Uncaptured}` carries the observation policy through the context, pipeline API,
shaders, texture creation and compute fast path. Existing-device and browser
contexts select uncaptured errors. Presentation and device provisioning remain
independent choices.

Matching-format requests reuse the cached Arc before native-texture admission.
Storage sRGB requests use a linear descriptor format while retaining the original
requested format in errors. The three ComputePass callers propagate the typed
error through their existing generic error boundary.

## Validation

Hosted validation is pending for commit `c8eccb6706c160c21b337c49b401b2f2737513a8`.
The workflow retains its Linux, Windows, fixture-exporter and positive-execution
gates. No tests, cases, helpers, assertions or doctests are added; two predicates
in the existing borrowed-context test now select the observation enum.

The obsolete 2D texture file-debt entry is removed following the hosted checker's
below-limit diagnostic. The cube ceiling remains 862; no ceiling is raised.

## Implementation context

Independent Code, Rule and Comment source reviews cover the implementation and
connected callers. Fresh reviews of the composed error model found no actionable
issues. The final model removes boxed wrappers, duplicate operation-specific
error variants, manual error implementations and unused conversion helpers.

Existing tests do not directly exercise the new view-error or polling-failure
branches; source review covers those branches. Public texture/cache/metadata and
device coherence, constructor and subresource validation, remaining migrations
and the complete audit remain open. This is an independent increment toward
#826, connected to #780 and #726.
