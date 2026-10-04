# fabelgeist-fbx

Binary FBX node, property, object, connection, and animation reading with a
pure-Rust inflater. `parse` and `Scene::parse` admit a
`fabelgeist_storage::StorageView` and return `FbxDecodeError`. Scene
interpretation remains the caller's responsibility; connection order determines
object and joint traversal order.

Binary framing keeps version, property count, array count, encoding code,
property tag, and storage positions in distinct nominal roles. The version owns
the 32-bit versus 64-bit header layout. Each node must end after its metadata
and name and within its containing node or file. Property counts are checked
against the declared block before allocation, and decoding must consume that
block exactly. Arrays admit plain or zlib encoding and require the declared
elements.

`FbxDecodeError` distinguishes format, checked-span, node-extent,
property-count, property-length, encoding, inflate, short-array, and unknown-tag
failures. It retains nominal rejected metadata, the affected section, and
concrete bounds or IO causes. Format the error at a presentation boundary;
internal code can inspect its variant and cause.

Numeric precision, signed zero, NaN bits, property order, raw string bytes,
lossy node-name decoding, and scalar booleans retain the reader's existing
behavior. Scalar booleans accept any nonzero byte; boolean arrays retain their
raw bytes, including values other than zero and one. Trailing decoded array
bytes remain ignored. Inflate retains only the declared prefix but consumes and
validates the complete stream, so corruption after that prefix still fails. Null
records and uninterpreted trailing file data retain their existing handling.

Node extents, property lengths, and unsupported array codes that were previously
tolerated now fail admission. Callers use the final view and concrete-error APIs
directly. Numeric property values and animation quantities still have primitive
interfaces recorded as migration debt.

`FbxObjectId` owns the signed object-table identity through objects,
connections, lookup, animation targets, and rig loading. It does not prove that
an object exists or that an identity is unique. Object records stay in file
order; lookup selects the last duplicate. `FbxObjectId::SCENE_ROOT` is the
connection destination for root children. Source-zero connections remain
ignored, unresolved sources remain in the graph, and child traversal skips them.
Integer and boolean identity properties retain their existing admission; float
and string properties do not become identities.

`FbxConnectionProperty` admits a connection's string or raw bytes and preserves
its lossy UTF-8 spelling, including unknown names. It classifies recognized
model transforms as `TransformProperty` and scalar components as `CurveAxis`.
Animation consumes these roles, preserving first-layer selection, connection
order, last-axis assignment, and key/value truncation. Unknown properties remain
uninterpreted. These types do not add object-existence, finite-value, or
nonzero-identity validation. Numeric property arrays and animation time/value
interfaces still require migration.

`FbxRecordName` and `FbxClassName` distinguish record and subclass identities.
Their decoded spelling remains exact after lossy UTF-8 admission, including
unknown names, namespaces, suffixes, and whitespace. Node and scene selectors
accept those identities directly. `ModelRole` classifies Model records as joint
candidates, nulls, or uninterpreted models. This does not establish rig
membership.
The former boolean object predicates are removed. Admitted scene records are
immutable, so callers cannot invalidate object lookup indices. `Scene::objects`
and `Scene::roots` expose records through borrowed iterators.

`FbxPropertyName` is an exact borrowed byte identity for `Properties70` keys.
Property lookup keeps the first table and first matching entry, without lossy
decoding or normalization. Byte constructors can address an uninterpreted key
that is not UTF-8; it cannot be confused with a record or class selector. Known
transform and axis keys have one shared owner used by both property lookup and
connection admission. Numeric scalar and whole-vector fallback behavior is
unchanged. Record-name encoding returns a shared storage view directly to wire
serializers, keeping selector identities out of primitive domain helpers.

`FbxQualifiedName` retains namespaces while `FbxObjectName` removes the last
namespace prefix. Both stop at the first NUL/one separator and use the same
lossy UTF-8 admission. Empty, missing, nontext and unfamiliar names keep their
previous handling. Object provenance and animation take labels retain the
normalized identity; animation targets retain the qualified identity. Neither
can be exchanged implicitly. A target rig constructor receives the normalized
label's nominal storage view, rather than using presentation text for identity.

The framing follows Blender's [binary FBX research](https://code.blender.org/2013/08/fbx-binary-file-format-specification/)
and [binary exporter](https://github.com/blender/blender-addons/blob/main/io_scene_fbx/encode_bin.py).
They describe the node metadata and plain/zlib array encodings; the exporter
selects 64-bit node headers from version 7500. No new version-range restriction
is imposed by this reader.
