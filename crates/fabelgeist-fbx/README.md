# Binary FBX header layout

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
a bespoke FbxArrayEncodingCode before passing it into the shared array decoder.
This keeps encoding identity distinct from counts, byte lengths and property
tags through all five array kinds: 32-bit and 64-bit floats, 32-bit and 64-bit
integers, and Boolean bytes.

The encoding code is open. Zero selects an uncompressed payload; every nonzero
value uses the existing Zlib decompression path, including values other than 1.
This preserves the current decode policy rather than asserting which codes the
wider format supports. Decoder errors, empty counts, extra decoded bytes and the
original float bit patterns and Boolean byte values remain unchanged. Offsets,
node property counts and structured decode errors remain separate migration
responsibilities.

A regression fixture records 95 observations from the unchanged main public
parser. Independently encoded wire properties cover all five array kinds with
zero, one and other nonzero codes; empty, ordinary, short and extra payloads;
and corrupt compressed streams. The fixture retains exact floating-point bit
patterns and Boolean bytes when comparing decoded output.

FBX array metadata stores element count, encoding and stored payload byte length
as three little-endian 32-bit words. Each bespoke owner decodes its own word
directly from a fixed-size byte array. `FbxArrayEncodingCode` remains typed
through the array decoder. Blender's public reader uses `<III` for this same
wire header. That reference establishes field width and order; this reader
retains its existing zero/raw and every-nonzero/Zlib policy.

[Blender FBX reader, read_array_params](https://github.com/blender/blender-addons/blob/main/io_scene_fbx/parse_fbx.py)

## Array counts and payload lengths

`FbxArrayCount` carries the declared element count from the header reader into
array decoding. `FbxArrayPayloadLength` represents the stored payload's byte
length, including compressed bytes when Zlib encoding is selected. The two
types distinguish element cardinality from byte length.

All encoded counts and lengths remain admitted. Their native `usize`
projections occur at buffer slicing, capacity, length-check and iteration
boundaries. The native array decoder determines the element width used with
the count. These conversions retain the existing host-size casts and
multiplication; they add no allocation limit or overflow policy.

## Property tags

The binary FBX model-file reader converts each one-byte property tag into the
private `FbxPropertyTag` enum before dispatch. Its thirteen variants distinguish
scalar numbers and Booleans, numeric and Boolean arrays, strings, and raw byte
payloads. The derived `TryFrom<u8>` rejects unsupported bytes at this boundary,
so property selection is exhaustive and cannot receive an unknown tag.
Rejected bytes retain the parser's existing character diagnostic.

Native little-endian scalar decoders determine their input-array widths.
Array decoding uses the same fixed-width input contract, so a tag's selected
decoder determines each element's width without a separate numeric argument.
Boolean truth rules, array Boolean bytes, string encoding, read ordering and
parser errors remain unchanged. Header layouts, array metadata, node property
counts, offsets and structured parser errors have separate responsibilities.

A regression fixture records 284 observations from the unchanged main public
parser. Independently encoded properties cover all 256 native tag bytes, all
thirteen known payload representations, missing and short payloads, and mixed
forward/reverse property sequences. Comparisons retain exact floating-point bit
patterns, invalid string bytes and the distinct string/raw property variants.

## Remaining decoding audit

These bespoke types do not complete the parser's other responsibilities.
`read_node` still carries native offsets and node property counts, and
`read_props` receives the latter. Those values and the parser's errors need
their own complete producer/consumer migrations. The header, array metadata
and property-tag types introduce no stricter framing, encoding, version,
allocation or error policy.
