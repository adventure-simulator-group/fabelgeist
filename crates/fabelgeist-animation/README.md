# fabelgeist-animation

Engine-native skeletons, keyframed animation clips, and retargeting between
rigs.

## Clip identity

`animation::AnimationClipName` is the bespoke type for an authored clip name.
`Animation.name` stores that type, and `Animation::new` requires it explicitly:

```rust
use fabelgeist_animation::animation::{Animation, AnimationClipName};

let clip = Animation::new(AnimationClipName::from("namespace:walk"));
```

Construct a name from `&str` or `String` at a native text boundary. Every string
is admitted unchanged, including the empty default, Unicode, whitespace,
control characters, and NUL. There is no trimming, normalization, or name
validation policy. Serde encodes the name as a scalar JSON string in the clip's
existing `name` field. `Display` and `Debug` preserve native string formatting.

Cloning and retargeting retain the typed name. Keep it as `AnimationClipName`
through clip storage and transfer; use serialization or formatting where a
native representation is needed. Joint tracks and joint queries are separate
roles and cannot accept a clip name directly.

## Joint requirements

[Joint requirements](joint-requirements.md) describes required and optional
retarget bindings, public construction, and their Boolean JSON representation.

## Retarget strictness

[Retarget strictness](retarget-strictness.md) describes permissive and strict
transfer policies, required-joint precedence, and the Boolean JSON boundary.

## Rig-profile labels

[Rig-profile labels](rig-profile-names.md) describes the typed labels retained
through profile construction and resolution, their string JSON representation,
and their separate role from clip and joint names.

## Retarget-profile labels

[Retarget-profile labels](retarget-profile-names.md) describes labels for
source-to-target retargeting recipes, their typed construction and retained
string representation.

## Rig inference

[Rig inference](rig-inference.md) describes how joint spellings are recognized,
the resulting profile and reports, and the limits of heuristic matching.
