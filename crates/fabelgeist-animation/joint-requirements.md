# Joint requirements

A rig profile maps skeleton joint names to humanoid roles. Each binding carries
`JointRequirement::Optional` or `JointRequirement::Required` in its `required`
field. A missing optional role appears in `ResolvedRig.missing`. A missing
required role rejects the rig before chain and root resolution.

The type belongs to `animation::retarget::profile` and is re-exported from
`animation::retarget`. Both roles are valid. `JointBinding::new` and
`RigProfile::with` create optional bindings; the existing fluent `required` and
`with_required` constructors select required bindings.

```rust
use fabelgeist_animation::animation::retarget::{
    HumanoidJoint, JointBinding, JointRequirement, RigProfile,
};

let mut pelvis = JointBinding::new("hips").with_alias("pelvis");
pelvis.required = JointRequirement::Required;
let profile = RigProfile::new("my rig")
    .with_joint(HumanoidJoint::Pelvis, pelvis)
    .with(HumanoidJoint::Head, "head");
```

Keep the requirement typed through binding construction, profile storage and
resolution. Direct field assignment requires a named enum variant. Standard
`From<bool>` and `From<JointRequirement>` conversions serve native Boolean
boundaries, including serde; ordinary profile logic uses the enum.

The `required` JSON field stays Boolean: `false` selects optional and `true`
selects required. Omitting the field selects optional. Other JSON types are
rejected with the native Boolean decoder's diagnostics. Cloning, equality and
round trips retain the role. Derived `Debug` names the enum variant.

Names are tried in alias order. For each alias, exact matching precedes
normalized matching. An exact skeleton name can therefore beat an earlier
normalized skeleton name, while an earlier matching alias still beats a later
alias. Identical duplicate skeleton names select the first joint. Missing
required roles are reported in profile insertion order.

Mixamo's profile carries the same twelve required body bindings, ten optional
body bindings and thirty optional finger bindings. Inferred pelvis bindings
remain required. `RetargetSettings.strict` is a separate Boolean policy: after
both rigs resolve, it rejects source roles absent from the target rig. It does
not determine whether a named skeleton joint is required.
