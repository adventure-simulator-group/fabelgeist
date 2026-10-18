# Binary FBX reader

## Record and class selectors

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

## Array encoding

The binary FBX model-file reader admits the four-byte array encoding field into
`FbxArrayEncoding` before passing it into the shared array decoder. This bespoke
enum keeps encoding identity distinct from counts, byte lengths and property
tags through all five array kinds: 32-bit and 64-bit floats, 32-bit and 64-bit
integers, and Boolean bytes.

This reader supports exactly two encoding codes: 0 selects `Uncompressed`, and 1
selects `Zlib`. Every other code returns `UnsupportedFbxArrayEncoding` before
payload decoding. Its public `reported_code` field retains the raw wire value;
callers can inspect this bespoke error through the parser's `anyhow::Error`.
This deliberately replaces the previous policy that treated every nonzero code
as Zlib. The shared decoder matches the two supported variants exhaustively.

The existing regression fixture records 95 observations through the public
parser. Independently encoded wire properties cover all five array kinds with
codes 0, 1, 7 and 4294967295; empty, ordinary, short and extra payloads; and
corrupt compressed streams. The 45 observations for supported encodings retain
their exact output, including floating-point bit patterns, Boolean bytes and
decoder errors. The other 50 observations now report the unsupported encoding
code, including empty arrays and corrupt payloads.

The native `Reader::u32` call reads the serialized encoding word: FBX array
metadata stores element count, encoding and compressed byte length as three
little-endian 32-bit words. The enum declares codes 0 and 1 as explicit `u32`
discriminants and derives `TryFrom<u32>` with `num_enum::TryFromPrimitive`.
The enum's custom-error configuration makes the standard `TryFrom` associated
error `UnsupportedFbxArrayEncoding` for every caller. Its derived `From<u32>`
constructor retains the rejected wire value. `read_props` checks the wire word
once and propagates that error directly with `?`; the enum remains typed through
array decoding. Blender's published binary format description documents code 0
for raw contents and code 1 for Zlib; it reports that other values had not been
observed. Those values are unsupported by this reader, without asserting that
Autodesk formally declares them invalid.

[Blender FBX binary format description](https://code.blender.org/2013/08/fbx-binary-file-format-specification/)
