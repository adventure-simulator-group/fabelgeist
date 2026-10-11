# Retarget joint lookup

A rig profile describes how to recognize a skeleton's joints. Its bindings
list joint spellings for humanoid roles, such as `Pelvis` or `Head`. Resolution
looks up those spellings in the skeleton and returns joint indices; animation
transfer then uses those indices.

For each spelling, an exact match wins. If there is no exact match, loose lookup
uses the last segment after `:` or `|`, keeps only ASCII letters and digits, and
lowercases ASCII letters. Thus `exporter:Left_Up_Leg` and `leftupleg` match.
Digits and side words or letters remain significant: `hips2` differs from
`hips`, and `leftarm` differs from `arm` and `rightarm`.

Loose keys can collide. `HIP-S` and `hi_ps` share a key; entirely non-ASCII or
punctuation-only spellings share the empty key. The first skeleton joint wins
among loose collisions and identical exact spellings. A later exact match still
beats an earlier loose match for the same query. Empty spellings, Unicode and
control characters remain admitted without validation or rewriting of stored
names.

Aliases are tried in authored order. The first alias that resolves wins, even
if it succeeds loosely and a later alias has an exact match. Explicit chains
keep authored order and repeated joints, omitting unresolved entries. An
unresolved named locomotion root is allowed and resolves to no root. Marker
detection requires every marker to resolve; an empty marker list never detects.

The private `RetargetMatchKey` bespoke type owns this lossy equality policy in
`animation::retarget::resolve::match_key`. The normalized index and query carry
that type directly. It is separate from inference's `InferenceStem`, which
strips recognized side markers before keyword matching. It is also separate
from exact joint identity and the labels of rig or transfer recipes. A transfer
recipe describes the source-to-target retargeting policy.

Public profile construction and serialization continue to use authored joint
spellings. Resolution errors and reports retain those spellings. Required
source-joint errors precede required target-joint errors, which precede strict
transfer-policy errors. Roles, chains, missing and unclaimed joints keep their
existing order. Lookup does not alter clip names, stored tracks or motion.

The broader exact-joint-name migration still needs the accepted rig-name owner
integrated separately. Joint indices, lengths and native predicates remain
outside this change; this guide does not exempt them from future migrations.
