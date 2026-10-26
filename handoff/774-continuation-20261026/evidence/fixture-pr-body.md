The existing settled-drape fixtures declare packed skin indices for one joint,
but supply empty joint-name and joint-state tables. Skin transfer rejects that
inconsistent rig before the garment assertions run. This is the first failure
in [current main's hosted checks](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/39901718104).

Supply the canonical root name and one identity joint state in the existing
wearer fixture. The geometry, skin indices, weights, eight test cases and
thirteen assertions are preserved.

## Validation

- Independent Code, Rule and Comment Reviews and all thirteen scoped audit
  lenses found no remaining actionable findings on this exact source.
- [Hosted validation on af5d569c1a608e9de0367f1eec99a79ef2180949](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/40233264407)
  passed formatting, generated-binding and source gates, workspace and
  standalone lint checks, the full Creator contract group, tactical animation
  and audio, and animation-report consumers.
- Full CI remains red: the unchanged tactical equipment group passed 46 cases
  and failed `physical_fit_survives_hold_drop_and_placeholder_rebuild_then_refits_new_wearer`
  with a Bevy schedule panic. Later dependent gates did not run. This draft
  therefore does not claim complete hosted acceptance.
- No local project checks or new tests are part of this repair.

## Implementation context

This completes fixture metadata required by the shared packed-rig contract
introduced in #774, as a prerequisite to the #780/#826 continuation under #726.
The identity state stays at the native fixture boundary, preserving builds
without the optional offline character-creator dependencies. Source comparison
confirms the exact two-field replacement.
