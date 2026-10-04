# Rig joint identity

`RigJointName` owns an exact skeleton label shared by MHR admission, creator
fitting and export, animation tracks and profiles, and tactical equipment
loading and skin binding. Its private
representation preserves every admitted string, including empty labels,
whitespace, namespaces and unknown helper names. A label does not prove joint
existence or anatomical meaning. JSON remains a scalar string.

`RigJointOrdinal` identifies a slot in an ordered skeleton. It is distinct from
a skin-buffer slot, a parameter row, a vertex index or a joint label. Its
transparent JSON representation remains the numeric slot. `RigJointCount` owns
ordered traversal and permits empty rigs; an ordinal confers no range proof or
membership in a particular rig. Lookup selects the first
matching label when duplicates occur. `RigJointLookupError` retains the rejected
nominal query; callers preserve that cause instead of formatting it into an
internal generic error.

`RigJointPart` is the authored fragment vocabulary used to construct sided
labels and select skin. `RigSide` belongs to rig spelling; equipment placement
chooses its side separately. Fragment matching intentionally retains existing
substring selection. Skin-family matching includes the exact owner and every
label with its `_twist` prefix. Family-prefix selection covers complete thumb
or tongue groups. Those queries return `RigJointMembership`; none establishes
canonical anatomy or retargeting authority.

Source-specific normalization belongs to the source format. The optional `fbx`
feature converts a namespace-stripped `FbxObjectName` into a rig label at its
constructor. Runtime consumers do not enable that feature or depend on MHR's
tensor backend. FBX qualified animation targets remain a distinct identity.

`RigJointPrefix` owns exact prefix material and its application, membership and
removal policy. Prefix removal returns a retained joint label; it neither
normalizes other namespaces nor establishes anatomical membership.

Admit external glTF and Bevy names at loading/binding boundaries. Retain nominal
labels through internal maps and comparisons. Representation conversion belongs
to the Bevy `Name` constructor, transparent scalar serialization, or the private
animation matching and inference constructors. Their loose keys never replace
the exact label used by tracks, bindings, storage or diagnostics.
Device parameters encode ordinals at GPU setup; unused frame landmark slots
remain a distinct state until their zero word is written. Such encoding does
not authorize primitive indexes in internal helpers.

Type replacement must preserve source spellings, serialized strings, ordering,
duplicate precedence, unknown-label handling and skin-family policy. It does
not change geometry, physical units, fitting arithmetic, animation or tactical
authority. Retargeting, poses and auto-rigging retain these ordered slots; skin
slots and clip-track addresses have separate owners. Other label roles,
mathematical quantities and generic provider errors remain migration work.
