# Rig-profile labels

`RigProfileName` is the bespoke type for a rig profile's label.
`RigProfile.name` stores it, `RigProfile::new` requires it directly, and
resolution retains it in `ResolvedRig.profile`. The type belongs to
`animation::retarget::profile` and is re-exported from
`animation::retarget`.

```rust
use fabelgeist_animation::animation::retarget::{
    HumanoidJoint, RigProfile, RigProfileName,
};

let profile = RigProfile::new(RigProfileName::from("my rig"))
    .with_required(HumanoidJoint::Pelvis, "hips");
```

Construct the label from `&str` or `String` at a native text boundary. Every
string is admitted unchanged, including the empty default, Unicode,
whitespace, quotes, control characters and NUL. There is no trimming,
normalization, uniqueness or nonempty requirement. A label describes the
profile; it does not select a joint or identify a skeleton asset.

Serde keeps the profile's existing `name` field as a JSON string. Decoding a
profile still requires that field even though `RigProfile::default` has an
empty label. Missing, duplicate and invalid fields retain the native decoder's
diagnostics and field order. Display keeps the label's spelling; Debug keeps
the native string's quoting and escaping.

Keep the label typed through profile construction, cloning and resolution.
Builtin Mixamo profiles return the `Mixamo` label, and inferred profiles return
`inferred`. Joint matching, alias order, required-joint errors, chain and root
resolution, and retargeted motion do not depend on the label's spelling.
Required-joint diagnostics quote it; mapping reports display it unchanged.

`RetargetProfileName` labels a source-to-target retargeting recipe, stored in
`RetargetProfile.name` and `ResolvedProfile.name`. Its separate meaning and
`source -> target` spelling are described in
[Retarget-profile labels](retarget-profile-names.md).
AnimationClipName identifies a clip, while skeleton joint names select joints;
neither can be passed as a rig-profile label directly.

Use the public retarget API to inspect the retained label:

```rust
use fabelgeist_animation::animation::retarget::{RigProfile, RigProfileName};
use fabelgeist_animation::Skeleton;

let label = RigProfileName::from(String::from("namespace:rig"));
let profile = RigProfile::new(label.clone());
let resolved = profile.resolve(&Skeleton::new(Vec::new())).unwrap();
assert_eq!(resolved.profile, label);
```
