# Binary FBX reader

## Object names

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
