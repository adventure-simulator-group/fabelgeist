# Skeletal animation and retargeting

Skeleton joints, clip tracks, profile bindings, chains, roots, markers and test
rigs use the shared `fabelgeist_rig::RigJointName`. It retains the exact authored
label, including namespaces, whitespace, Unicode, empty labels and NUL. A label
does not prove anatomical meaning or membership in a skeleton. Profile labels,
transfer labels, clip labels and serialized humanoid-role spellings have
separate owners. JSON labels remain scalar strings.

`RigJointPrefix` owns exact prefix material. Mixamo's prefix operations preserve
the existing `mixamorig:` policy; another namespace is neither rewritten nor
classified as that prefix. The authored Mixamo hierarchy remains separate from
the humanoid role vocabulary.

Retargeting resolves an exact label first. Its private `RetargetMatchKey` then
permits a loose match: remove everything through the last colon or pipe, keep
ASCII alphanumerics, and lowercase them. Different exact labels can share that
key. Exact matches take precedence; duplicate labels and loose-key collisions
retain the first skeleton joint. Candidate binding order also remains
significant. Never use a matching key as the persisted or displayed identity.

Inference has its own side and keyword policy. It strips supported side markers,
uses a closed keyword vocabulary and prefers a longer matching keyword. Ties
retain skeleton order. Inference requires a pelvis only when it finds one;
unrecognized rigs can produce an empty profile. Inferred labels remain exact
joint identities when passed to resolution.

`JointRequirement` and `RetargetStrictness` express admission choices internally;
their existing serialized fields remain booleans. `RetargetError` distinguishes
missing required bindings from strict target-role omissions and retains ordered
roles and candidate labels. Parsing an unknown humanoid role returns
`UnknownHumanoidJoint` with its exact rejected spelling. Render messages through
`Display`; classify failures by their variants. `RetargetReport` owns rendered
mapping diagnostics and confers no joint-query authority.

`RigJointOrdinal` addresses ordered skeleton joints and `RigJointCount` owns
cardinality and traversal, including empty rigs. `SkeletonJoints` accepts only
that coordinate for indexing. `SkinJointOrdinal` addresses a separate skin
buffer; sparse slots retain identity matrices in their holes. An explicitly
ordered fallback can translate a rig slot into a skin slot when an asset has
no skin-slot mapping. Neither ordinal proves range validity or membership in a
particular skeleton.

`LocalPose` contains joint-local transforms; `ModelPose` contains accumulated
transforms. Their separate owners prevent accidental reuse across coordinate
spaces. Both retain array serialization and ordered rig-slot indexing. Parents
appearing later retain the existing root policy in pose accumulation. Supplied
rest poses of the wrong cardinality still fall back to the authored rest pose.
`SkeletonModelMatrices` and `SkinJointMatrices` retain the corresponding rig
and skin coordinate authorities through skinning. `SkinningError` reports
expected and actual rig counts while preserving the existing display message.

`RigJointChain` owns ordered rig slots, continuous `ChainPosition` interpolation
and nearest-joint selection. Empty chains retain identity rotation and the
existing slot-zero fallback; singleton chains retain their endpoint policy.
Clip-track addresses and prepared source-chain addresses are distinct from rig
slots. `VertexSkinWeights` keeps each vertex's four slots and blending quantities
in one binding, rather than parallel arrays. `SkinBlendWeight` preserves IEEE
values and the existing summation and normalization operations. These types do
not claim imported or computed weights are finite, nonnegative or normalized.
Encode primitive slot words and weight scalars only at SDK/payload boundaries.

Type admission does not change fitting arithmetic, interpolation, root motion,
proportions or hierarchy behavior. Animation time, physical quantities, math
kernels and remaining booleans still require connected migration through their
producers and consumers.
