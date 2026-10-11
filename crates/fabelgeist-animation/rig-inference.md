# Inferring a rig profile

A rig profile describes how to recognize a skeleton's joints. If an imported
skeleton has no authored profile, `RigProfile::infer(&skeleton)` supplies a
starting recipe by guessing humanoid roles from its joint spellings. Prefer an
explicit profile for a rig you will reuse, and inspect the inferred mapping
before relying on it.

```rust
use fabelgeist_animation::animation::retarget::{RigProfile, RetargetProfile};
use fabelgeist_animation::skeleton::Skeleton;

let skeleton = Skeleton::new(Vec::new());
let inferred = RigProfile::infer(&skeleton);
let transfer = RetargetProfile::new(inferred.clone(), inferred);
let resolved = transfer.resolve(&skeleton, &skeleton)?;
println!("{}", resolved.report(&skeleton, &skeleton));
# Ok::<(), anyhow::Error>(())
```

The source-to-target retargeting recipe above reports which roles are shared,
missing or unmapped. An inferred profile remains ordinary editable,
serializable profile data. It has the `inferred` rig-profile label, a TPose
reference and the skeleton's exact joint spelling in every selected binding.
An inferred pelvis binding is required; unrecognized roles remain absent.

## Recognition and ordering

Inference has a private matching vocabulary distinct from exact joint identity
and from rig-profile, retarget-profile and clip labels. Its bespoke types carry
the parsed side, a lossy ASCII stem and a recognized hint through every
recognition decision. Callers continue to pass a skeleton to public inference;
the private vocabulary is not another public naming API.

The parser keeps the final colon or bar namespace component. It recognizes
left before right when those words occur anywhere, then tries `l` or `r`
separator prefixes and suffixes, then a lone side letter before an ASCII
uppercase character. It removes separators and non-ASCII characters and
lowercases ASCII alphanumerics for matching. This normalization never rewrites
the joint spelling stored in the profile. Unicode-only or empty stems may have
no recognized role.

Spine hints match a prefix on center joints and claim those joints first. The
spine chain retains skeleton order, including extra segments beyond the four
named spinal roles. Body hints match containment; a longer recognized keyword
wins, and equal-length matches retain the first unclaimed skeleton position.
The ordered role catalog and center/left/right passes remain significant.

Finger hints also match containment, in catalog and skeleton order on each
side. At most three candidate segments are considered. Pinky and little hints
share the little-finger roles, so an earlier hint can occupy a slot before the
later hint considers it. Inference does not sort joint-number suffixes or
replace already occupied roles.

Only the exact normalized root, reference or armature hints select a dedicated
root, and only on a center joint with no parent. The first eligible skeleton
position wins. Otherwise locomotion uses the pelvis. A name containing a root
hint plus other characters is not an exact root hint.

## Limits and ownership

These heuristics preserve unusual existing matches and do not validate anatomy,
resolve ambiguity or prove skeleton membership. Inspect reports and author a
profile when a heuristic chooses an unsuitable role. Required-joint resolution
still precedes strict transfer-policy checks. Chain and root queries retain
their existing resolution behavior; inference adds no rejection rule.

Matching stems are not exact joint names. Exact joint-name migration depends on
the canonical rig owner and is separate from this private inference boundary.
RigProfileName, RetargetProfileName, AnimationClipName, JointRequirement and
RetargetStrictness keep their existing public construction and wire contracts.
The private matching vocabulary changes no JSON, Display, Debug, report or clip
format. Skeleton indexes, intrinsic token lengths and native membership
predicates retain their existing representations in this increment.
