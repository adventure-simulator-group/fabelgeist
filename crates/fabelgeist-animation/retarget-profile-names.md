# Retarget-profile labels

A rig profile describes how to recognize a skeleton's joints. A retarget
profile joins two rig profiles into a source-to-target retargeting recipe.
`RetargetProfileName` is the bespoke type carrying that recipe's label.
`RetargetProfile.name` stores it and `ResolvedProfile.name` retains it through
resolution. The type belongs to `animation::retarget::profile` and is
re-exported from `animation::retarget`.

```rust
use fabelgeist_animation::animation::retarget::{
    RetargetProfile, RetargetProfileName, RigProfile, RigProfileName,
};

let source = RigProfile::new(RigProfileName::from("source"));
let target = RigProfile::new(RigProfileName::from("target"));
let generated = RetargetProfileName::from_rig_labels(&source.name, &target.name);
let profile = RetargetProfile::new(source, target);
assert_eq!(profile.name, generated);

let authored = RetargetProfile {
    name: RetargetProfileName::from(String::from("namespace:my transfer")),
    ..profile
};
```

The two-rig constructor generates exactly `source -> target`. Its label is a
snapshot: renaming either rig afterward leaves the recipe label unchanged.
Two empty rig labels generate ` -> `, while `RetargetProfile::default` has an
empty label. The named factory accepts canonical `RigProfileName` values
directly; keep the result typed through storage, cloning and resolution.

Admit authored text from `&str` or `String` at a text boundary. Every spelling
is retained, including empty text, Unicode, whitespace, quotes, control
characters and NUL. There is no trimming, normalization, nonempty or uniqueness
policy. Rig labels and `AnimationClipName` have separate meanings and cannot
be used as recipe labels directly.

Serde retains the existing required `name` JSON string. Decoding still rejects
missing, duplicate and invalid fields with the native diagnostics and field
order. Omitted settings retain their existing defaults. Display prints the
spelling unchanged; Debug retains the inner string's quoting and escaping,
including inside profile and resolved-profile output.

Reports display the retained recipe label. Required source-joint errors still
precede required target-joint errors, followed by strictness checks. Joint
matching, alias and role order, root and chain resolution, and the complete
retargeted clip do not depend on this label's spelling.
