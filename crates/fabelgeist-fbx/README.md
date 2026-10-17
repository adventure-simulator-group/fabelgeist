# FBX record and class selectors

`FbxRecordName` identifies a decoded record such as `Geometry` or `Vertices`.
`FbxClassName` identifies an object subclass such as `Mesh` or `Cluster`. These
bespoke enums keep the two roles distinct in tree and scene queries. Recognized
protocol labels use variants such as `FbxRecordName::Geometry` and
`FbxClassName::Mesh`; admission classifies each decoded label once.

All other labels use `Unknown` with a distinct opaque payload for each owner.
Private, immutable payload spelling prevents constructing a recognized label as
unknown or exchanging unknown record and class labels. Native string and byte
constructors always pass through canonical admission. Empty labels remain
representable and supply the default when a subclass property is missing or has
another property variant.

Binary admission retains lossy UTF-8 decoding, exact case, unknown names,
whitespace, namespaces and suffixes. String properties (`Prop::Str`) and raw
byte properties (`Prop::Raw`) both supply subclass labels. No ASCII, nonempty
or strict-UTF8 restriction is introduced. Display and debug output retain the
exact decoded spelling, including quoted debug strings.

`Node::child` and `Scene::child_of_kind` select the first match. Ordered
iterator queries preserve repeated records and file order. An iterator borrows
its selectors while iterating; yielded records borrow the tree or scene and can
outlive the selector values. `FbxRecordName::encoded` supplies the decoded
name's UTF-8 bytes at a node-encoding boundary, including any replacement
characters. It does not recover undecodable original bytes. Domain selection
uses enum values rather than formatting or encoded bytes.

Record and subclass identity do not establish rig membership. Existing model
classification, property-table keys, object naming, connection IDs, numeric
payloads, animation timing and parser errors retain their current behavior.
