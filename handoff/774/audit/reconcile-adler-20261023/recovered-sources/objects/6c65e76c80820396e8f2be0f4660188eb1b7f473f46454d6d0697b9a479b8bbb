# Shared rig identity

`RigJointName` represents an exact authored skeleton label. MHR admission,
character fitting and export, tactical equipment, and animation metadata share
this lightweight crate; runtime consumers do not need the MHR tensor backend.

A label is distinct from its `RigJointOrdinal` in an ordered rig and from an
anatomical `RigJointPart`. Use `mhr::sided` for authored MHR side/part
construction, and `index_in` or `require_in` for lookup. Duplicate labels retain
the first matching ordinal. A missing lookup retains the exact requested label
in `RigJointLookupError`.

Admission retains empty, unfamiliar, namespaced, and whitespace-bearing labels.
The source format owns namespace removal or normalization before constructing
the label. Equality and ordering compare the exact spelling. Explicit
case-insensitive queries do not change identity or serialization.

The explicitly named `mhr` profile owns the authored world/pelvis, center and
sided naming convention; a proposed right-side partner still needs lookup in
its receiving rig. Anatomical fragment and exact family-prefix queries do not
prove anatomy. Clothing owns its `_twist` skin-family policy, including the
unconstrained helper suffixes; animation owns its mask classification.
The strum-authored canonical vocabulary does not restrict unfamiliar labels.

At external boundaries:

- Serde encodes labels as scalar strings, including glTF node names and saved
  body metadata. Internal collections retain `RigJointName`.
- Bevy `Name` admission constructs a label; its native string representation is
  created only when handing a label back to Bevy or parsing native text.
  Tactical scene admission caches the exact label, canonical animation role,
  body region, and pelvis classification once; sampling and equipment consume
  that metadata. The shared rig owner does not depend on Bevy.
- `RigJointTable` retains rig order and owns ordinal access. Guaranteed rig
  traversal uses ordinal indexing; unproven external ordinals use checked
  lookup. Ordinals do not assert membership in another rig. Uniform insertion
  and rigid skin assignment use checked `u32` encoding at their final packed
  word handoff.
- Skin masks convert membership to the existing zero/one GPU words. Unused
  frame landmarks retain their original zero padding.

These types describe identity and selection. They retain skeleton order, attachment parents, skin weights and animation
authority; owning application policies remain separate from shared identity.
