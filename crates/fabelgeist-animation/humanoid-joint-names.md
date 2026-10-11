# Humanoid role spellings

A humanoid role describes anatomy, such as `UpperArmLeft`. A rig's authored
skeleton might name that joint `mixamorig:LeftArm`. These names have different
owners: the role vocabulary helps match anatomy across rigs, while the authored
joint name identifies a joint within one skeleton.

`HumanoidRoleName` is the bespoke type for text at the role-classification
boundary. It preserves arbitrary text, including empty strings. It does not
assert that the text names one of the 53 canonical roles. `HumanoidJoint::name`
produces a typed canonical spelling; `HumanoidJoint::try_from` accepts the type
and checks exact, case-sensitive membership. There is no normalization or
namespace stripping at this boundary.

```rust
use fabelgeist_animation::animation::retarget::{HumanoidJoint, HumanoidRoleName};

let decoded = HumanoidRoleName::from(String::from("UpperArmLeft"));
let role = HumanoidJoint::try_from(decoded).unwrap();
assert_eq!(role, HumanoidJoint::UpperArmLeft);
```

`UnknownHumanoidJoint` retains the rejected typed spelling through `name()` and
implements `std::error::Error` without a nested cause. Its Display diagnostic
and String-style Debug quoting, escaping and formatting flags are preserved. The
standard `FromStr` adapter admits native text once and returns the same error.
The name's Display and Debug use native string formatting; the role enum's
Display retains its unpadded complete spelling.

The Rust API now returns and accepts the bespoke spelling type directly. Raw
`String` and rig-profile labels cannot be classified without explicit admission
at the text boundary. The former `as_str` and `from_name` methods are removed.
There is no raw-text extraction API, compatibility facade or extra serialization
format. `HumanoidJoint` serialization, map keys, catalogue order and
relationships remain unchanged. This change does not introduce an exact
skeleton-joint identity type or change the separate `HumanoidChain` vocabulary.
