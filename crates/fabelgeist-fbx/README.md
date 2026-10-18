# Binary FBX reader

## Header layout

The binary FBX reader admits the four-byte file version into `FbxVersion` before
selecting node-header layout. Version 7500 changes the end offset, property
count and property-list byte length from 32-bit to 64-bit fields. A one-byte
name length follows those three words, giving 13-byte and 25-byte records and
null sentinels. `HeaderFormat` calculates the widths from those field types.

Blender's FBX reader implements the two layouts as `<IIIB` and `<QQQB` and
selects the wider layout at version 7500. Its binary signature also establishes
the four-byte version's location immediately after the signature.
[Blender FBX reader](https://github.com/blender/blender-addons/blob/main/io_scene_fbx/parse_fbx.py)

The version remains an open wire identity: every `u32` is admitted, including
values outside the documented 7100–7700 range. Layout selection preserves the
existing cutoff without claiming support for other features. Derived `From` and
`Into` provide native decoding and fixture-serialization ports. The parser still
checks only the original magic prefix; its two following marker bytes remain
ignored. Naming the full signature's width does not strengthen that validation.

Named version and record scenarios encode the same 59 public-parser observations
as the original golden fixture. The version uses the shared `FbxVersion` owner.
Independent narrow/wide wire encoders and named wire-header and node records
describe serialized fields before reader admission. Their native offsets, byte
lengths and counts are fixture-encoding ports, not parallel production semantic
owners. The expected snapshot remains unchanged.

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

## Remaining decoding audit

Header-layout and array-encoding ownership do not complete the parser's other
responsibilities. `read_node` still carries native offsets and property counts;
`read_props` and `decode_array` still carry property tags, array counts and
scalar widths. Scalar conversions still contain guaranteed-length `unwrap`
calls. Those values and the parser's errors need their own complete
producer/consumer migrations. Header ownership adds no stricter framing,
version or allocation policy; unsupported array encodings follow the checked
admission described above.
