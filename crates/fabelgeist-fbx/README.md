# Binary FBX reader

## Property-table keys

`FbxPropertyName` identifies a key in an FBX (Filmbox) `Properties70` block,
such as `RotationOrder` or `Lcl Translation`. Node lookup, animation defaults
and time spans, and the Momentum Human Rig (MHR) character importer use this
shared bespoke type.

Keys borrow exact bytes from string or raw-byte properties. Lookup performs no
UTF-8 decoding, normalization or case folding. Unknown names, embedded
separators and undecodable bytes remain distinct. Authored text enters through
an explicit conversion; known importer keys have shared constants.

Lookup selects the first `Properties70` block and its first child with a
matching string or raw-byte key. The child record name is not checked. Numeric
values start at property index 4. A missing or nonnumeric scalar uses the
caller's default; a missing, short or partly nonnumeric vector uses the entire
default vector. Selected nodes borrow the tree and can outlive temporary keys.

A property-table key does not classify an animation connection or establish the
type of its value. Record and subclass labels, connection IDs and labels,
numeric payloads, parser errors and rig membership retain their existing owners
and behavior.

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
