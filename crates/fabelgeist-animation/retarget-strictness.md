# Retarget strictness

`RetargetSettings.strict` carries `RetargetStrictness::Permissive` or
`RetargetStrictness::Strict`. Permissive transfer retargets shared humanoid
roles and ignores resolved source roles absent from the target. Strict transfer
rejects those omissions, including resolved roles unused by the current clip.
Extra target roles are allowed under either policy.

The type belongs to `animation::retarget::profile` and is re-exported from
`animation::retarget`. Settings and new profiles default to `Permissive`.
Choose the named policy directly:

```rust
use fabelgeist_animation::animation::retarget::{
    RetargetSettings, RetargetStrictness,
};

let settings = RetargetSettings {
    strict: RetargetStrictness::Strict,
    ..Default::default()
};
```

Keep the policy typed through settings, profile storage, resolution and the
retargeter's settings accessor. Direct field assignment requires the enum.
Standard `From<bool>` and `From<RetargetStrictness>` conversions serve native
Boolean boundaries, including serde. The existing `strict` JSON field remains
Boolean: false selects permissive, true selects strict, and omission selects
permissive. Other JSON types are rejected by the native Boolean decoder.
Successful settings and profile round trips retain the policy. Derived Debug
names the selected variant.

[Joint requirements](joint-requirements.md) govern missing named skeleton
joints separately. Required source-joint errors precede required target-joint
errors, and both precede strictness. Optional unresolved source roles are
reported without becoming strict omissions. Missing explicit chain names are
filtered; a missing named root remains allowed.

Strict omissions appear in source profile insertion order. Shared transfer
roles retain the canonical humanoid order. Matching still tries aliases in
order, with exact matching before normalized matching for each alias and the
first joint winning identical duplicate names. The policy does not alter
retargeting arithmetic or clip output when the same roles are shared.
