Animation catalog builders receive a checked `MotionId`, then convert it to
`String` when a pack lacks that motion. Store the existing `MotionId` in
`CatalogError::UnknownMotion` at both pose and reference error producers so
the failure retains its admitted identity.

Invalid-motion-stem and invalid-directory rejections continue to carry the
offending raw input. Related: #780 and #726.

## Validation

Source inspection traced both error producers and their diagnostic consumers.
Independent Code, Rule and Comment Reviews and required hosted checks are
pending. No tests, cases, helpers, assertions or doctests were added.

## Implementation context

The error reuses the canonical animation `MotionId`; both producers clone
the supplied identifier. This addresses the inherited motion-error identity
finding without adding a second identity owner or compatibility API.
