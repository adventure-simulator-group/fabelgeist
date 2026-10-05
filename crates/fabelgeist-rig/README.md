# Shared rig identity

`RigJointName` represents an exact authored skeleton label. MHR admission,
character fitting and export, tactical equipment, and animation metadata share
this lightweight crate; runtime consumers do not need the MHR tensor backend.

A label is distinct from its `RigJointOrdinal` in an ordered rig and from an
anatomical `RigJointPart`. Use `RigJointName::sided` for authored side/part
construction, and `index_in` or `require_in` for lookup. Duplicate labels retain
the first matching ordinal. A missing lookup retains the exact requested label
in `RigJointLookupError`.

Admission retains empty, unfamiliar, namespaced, and whitespace-bearing labels.
The source format owns namespace removal or normalization before constructing
the label. Equality and ordering compare the exact spelling. Explicit
case-insensitive queries do not change identity or serialization.

Skin selection returns `RigJointMembership`. Exact owners and their `_twist`
prefixes retain the existing authored selection policy, including unconstrained
suffixes. Anatomical fragment selection and family-prefix selection are
separate queries. MHR topology queries recognize world/pelvis and center or
sided label families; a proposed right-side partner still needs lookup in its
receiving rig.

At external boundaries:

- Serde encodes labels as scalar strings, including glTF node names and saved
  body metadata. Internal collections retain `RigJointName`.
- Bevy `Name` admission constructs a label; its native string representation is
  created only when handing a label back to Bevy or parsing native text.
- Native array indexing and glTF/GPU joint words explicitly convert an ordinal.
  Ordinals do not assert bounds or membership in another rig.
- Skin masks convert membership to the existing zero/one GPU words. Unused
  frame landmarks retain their original zero padding.

These types describe identity and selection. They do not change skeleton order,
attachment parents, skin weights, fitting geometry, or animation authority.
