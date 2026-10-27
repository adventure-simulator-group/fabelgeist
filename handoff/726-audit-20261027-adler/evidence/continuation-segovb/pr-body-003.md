Whole-texture format-view requests currently erase native validation causes into
text, panic when a matching-format cached view is absent, and ignore failed
device polling. `Texture2d`, `Texture3d` and `TextureCube::view_with_format` now
return a shared `TextureViewResult` preserving the requested format and native
cause. Missing resources return typed errors, and failed observation returns a
typed progress error before caching a new view.

The shared `ValidationScope` keeps its device and guard together, attempts
polling before always popping the scope, and retains both polling and captured
validation errors. Alternate-format views honor the borrowed-context policy.
Cached Arc identity, cache-first precedence, storage sRGB projection,
descriptors and the three ComputePass error recipients retain their existing
behavior.

The workflow preserves the existing Linux, Windows, fixture-exporter and
positive-execution gates. At final commit
`961d226be23a81d9731a11958d62fe8a04b93e81`, [hosted run
40372060212](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/40372060212)
has passed the existing 13 GPU library tests, two GPU doctests, exact
borrowed-context case, ten Windows foliage cases, 67 constraint/shell library
tests and five constraint doctests, with zero failures or ignored cases in those
populations. Formatting, generated bindings, semantic/growth checks and Clippy
have also passed. The remaining Linux checks are running; this PR remains a
draft.

The hosted baseline supports only two relevant ceiling reductions: 2D texture
538 to 502 and cube texture 899 to 862. No ceiling is raised. No tests, cases,
helpers, assertions or doctests are added; the saved WIP test/fixture files and
new doctests are excluded because they are absent from current main.

## Implementation context

Independent Code, Rule and Comment source reviews checked the scoped producers,
shared observation owner, unchanged receiving bodies, native SDK contracts and
workflow composition. They found no new scoped source defect after the baseline
repair. Existing tests do not directly exercise the new view-error or
polling-failure branches; their behavior is supported by source traces. Required
checks must pass at the final revision before this PR is marked ready.

This is an independent increment toward #826, connected to #780 and #726. Public
texture/cache/metadata/device coherence, constructor and subresource validation,
remaining migrations and the complete audit remain open.
