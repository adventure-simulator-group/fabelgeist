Whole-texture format-view requests currently erase native validation causes
into text, panic when a matching-format cached view is absent, and ignore
failed device polling. This change gives `Texture2d`, `Texture3d` and
`TextureCube::view_with_format` a shared `TextureViewResult` that preserves
the original requested format and concrete native cause. Missing resources
return typed errors, and failed observation returns a typed progress error
before publishing the new view.

The shared native `ValidationScope` keeps the device and guard together,
attempts polling before always popping the scope, and retains both polling
and captured validation errors. Alternate-format views honor the existing
borrowed-context policy. Cached Arc identity, cache-first precedence, storage
sRGB projection, view descriptors and the three ComputePass error recipients
retain their existing behavior.

Validation is pending on this draft. The workflow preserves the current Linux,
Windows, fixture-exporter and positive-execution gates, and adds source-pinned
evidence for the existing 13 GPU library tests, two doctests and exact borrowed
context case. It also exports the current growth baseline so the relevant
reductions can be applied before final validation. No tests, cases, helpers,
assertions or doctests are added. The saved WIP's added test/fixture files and
new doctests are excluded because they are absent from current main.

## Implementation context

Independent Code, Rule and Comment source reviews checked the scoped producers,
shared observation owner, unchanged receiving bodies, native SDK contracts and
workflow composition. Source review found no new correctness defect; Rule
Review retains the hosted baseline reductions as unfinished. Hosted checks and
the final revision review remain required. Existing tests do not directly
exercise the new view-error or polling-failure branches, so their behavior is
currently supported by source traces rather than runtime coverage.

This is an independent increment toward #826, connected to #780 and #726.
Public texture/cache/metadata/device coherence, constructor and subresource
validation, remaining migrations and the complete audit remain open.
