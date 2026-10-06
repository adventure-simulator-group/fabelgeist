# FBX record and class selectors

`FbxRecordName` identifies a decoded record such as `Geometry` or `Vertices`.
`FbxClassName` identifies an object subclass such as `Mesh` or `Cluster`. The
bespoke types keep the two roles distinct in tree and scene queries.

Binary admission retains lossy UTF-8 decoding, exact case, unknown names,
whitespace, namespaces and suffixes. String properties (`Prop::Str`) and raw
byte properties (`Prop::Raw`) both supply subclass labels. A missing property or
any other property variant yields an empty label. Constants supply shared
spellings for standard records and subclasses; unknown labels remain queryable
without normalization or grammar validation.

`Node::child` and `Scene::child_of_kind` select the first match. Ordered
iterator queries preserve repeated records and file order. An iterator borrows
its selectors while iterating; yielded records borrow the tree or scene and can
outlive the selector values. `FbxRecordName::encoded` supplies the decoded
name's UTF-8 bytes at a node-encoding boundary, including any replacement
characters. It does not recover undecodable original bytes.

Record and subclass identity do not establish rig membership. Existing model
classification, property-table keys, object naming, connection IDs, numeric
payloads, animation timing and parser errors retain their current behavior.
