*Written by Codex on behalf of @segovb.*

A context wrapping an existing GPU device selects `Uncaptured`, but 3D and
cube texture construction and context-bearing cube-face selection still
opened a validation scope and waited for device progress. Those operations
could wait on unrelated submissions from the device provider.

Route all three texture constructors and cube-face selection through the
shared `ValidationScope`. `Uncaptured` creates no validation scope or
validation-only wait. `BlockingScopes` always finishes the scope, preserving
polling failures and captured validation errors together. This also fixes 2D
construction discarding polling failures.

Cube-face APIs now return `TextureViewResult<TextureView>`. Observed failures
carry the selected `CubeFace`, original requested format and structured cause.
Face selection lives beside whole-texture selection in the existing view
module. Context-free face selection and browser targets omit synchronous
observation; readback and submission backpressure retain their required waits.

Closes #989. Related: #826.

## Validation

Independent Code, Rule and Comment Reviews found no actionable source defects.
[Hosted checks](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/40420900203)
are running at `f2978ded5`; the relevant cube baseline reduction and final
acceptance remain pending. No tests, cases, helpers, assertions or doctests
were added. The issue is supported by source inspection; a hang was not
reproduced.

## Implementation context

The shared validation policy and scope-completion owner introduced in #988
are reused directly. Texture constructors retain their generic error boundary;
cube-face selection uses the existing bespoke view error owner with a face
payload. The owning GPU guide and context documentation describe the covered
operations and independent completion waits.

