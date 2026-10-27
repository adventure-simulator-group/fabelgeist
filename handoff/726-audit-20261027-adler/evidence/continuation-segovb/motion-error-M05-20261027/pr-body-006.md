Animation catalog builders receive a checked `MotionId`, then convert it to
`String` when a pack lacks that motion. Store the existing `MotionId` in
`CatalogError::UnknownMotion` at both pose and reference error producers so
the failure retains its admitted identity.

Invalid-motion-stem and invalid-directory rejections continue to carry the
offending raw input. Related: #780 and #726.

## Validation

Independent Code, Rule and Comment source reviews found no actionable
findings on `ff43d1818`. The
[hosted run](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/40436696284)
passed formatting, Clippy with warnings denied, existing animation, rig,
geometry and terrain checks, GPU validation, and Windows portability.
Required execution artifacts were checked against this revision.
The terrain logs include 14 ignored cases.
No tests, cases, helpers, assertions or doctests were added.

## Implementation context

The error reuses the canonical animation `MotionId`; both producers clone
the supplied identifier. Source inspection covers both producers and the
catalog-construction diagnostics that receive their errors.
