# Binary FBX reader

## Array cardinality and layout

The binary FBX model-file reader admits each encoded array element count into
FbxArrayCount and passes it through the shared decoder. FbxArrayKind names the
five supported element layouts: 32-bit and 64-bit floats, 32-bit and 64-bit
integers, and Boolean bytes. The count stays distinct from byte lengths, and
numeric strides no longer cross the decoder interface.

Explicit native adapters project the count and layout at allocation,
length-check, iteration, slice and diagnostic ports. Raw data is still copied
before its decoded-length check. Compressed data still reserves capacity before
inflation. Host-sized multiplication, overflow and allocator behavior are
retained; no resource limit or checked framing admission is added. All encoded
counts, including zero, remain admitted. Boolean arrays retain every byte value,
and extra decoded bytes remain ignored.

A regression fixture records 282 observations through the public parser. Independent native records cover both header layouts, all five element
layouts, zero/one/two/three declarations, raw and Zlib encodings, unsupported
encoding codes, short and extra payloads, corrupt compressed input, exact float
bits and Boolean bytes. Large raw declarations fit even 32-bit native multiplication; extreme
compressed capacities are not executed. No 32-bit runtime or allocator-safety
verdict is claimed.

Property counts, property tags, array encoding, version/header layout, byte
framing and structured errors remain separate migration responsibilities.
Other pending increments should retain each final owner. Checked array encoding
is documented below.

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
