# FBX object names

The binary FBX reader carries two bespoke object-name types.
`FbxQualifiedName` retains namespaces, while `FbxObjectName` keeps the suffix
after the last colon for Momentum joint matching. Both exclude the first
NUL-one class separator and everything following it. Source bytes are decoded
with replacement for invalid UTF-8 after that cutoff.

The private fields admit empty, unknown and whitespace-bearing spellings
without inventing a label grammar. Qualified names are constructed from source
bytes; normalized names are constructed from qualified names. Neither exposes
mutable text or an unchecked string constructor. Missing or nontext name
properties yield empty names, and both string and raw-byte properties retain
their existing decoding behavior. Display and Debug preserve the decoded text.

`Object` retains both owners. Animation node targets retain qualified names,
and take names retain normalized names. MHR skeleton admission converts a
normalized name directly into the shared `fabelgeist_rig::RigJointName`;
blend-shape metadata retains `FbxObjectName`. The shared rig crate remains
independent of the FBX reader and tensor runtimes.

These types do not change object IDs, graph resolution, joint ordering,
transform arithmetic, curve sampling or malformed-record handling. Record and
class selection and other parser policies remain independent of these name
identities.
