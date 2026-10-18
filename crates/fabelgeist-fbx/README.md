# Binary FBX reader

FBX is a format for 3D model files. This crate reads its binary node and
property tree. The decoder returns a bespoke `FbxDecodeError` through
`fabelgeist_fbx::Result<T>`; see [binary decoding errors](binary-errors.md) for
classifications, typed context and native causes. Scene queries built from
already decoded roots remain infallible.

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
`FbxArrayEncoding` before passing it into the shared array decoder. This keeps
encoding identity distinct from counts, byte lengths and property tags through
all five array kinds: 32-bit and 64-bit floats, 32-bit and 64-bit integers, and
Boolean bytes.

This reader supports exactly two encoding codes: 0 selects `Uncompressed`, and 1
selects `Zlib`. Every other code returns `UnsupportedFbxArrayEncoding` before
payload decoding. Its public `reported_code` field retains the raw wire value;
the decoder retains it as the source of
`FbxDecodeError::UnsupportedArrayEncoding`.
The shared decoder matches the two supported variants exhaustively.

The existing regression fixture records 95 observations through the public
parser. Independently encoded wire properties cover all five array kinds with
codes 0, 1, 7 and 4294967295; empty, ordinary, short and extra payloads; and
corrupt compressed streams. The 45 observations for supported encodings retain
their exact output, including floating-point bit patterns, Boolean bytes and
decoder errors. The other 50 observations report the unsupported encoding code,
including empty arrays and corrupt payloads.

FBX array metadata stores element count, encoding and stored payload byte length
as three little-endian 32-bit words. Each bespoke owner decodes its own word
from a fixed-size byte array. `FbxArrayEncoding::try_from_le_bytes` uses the
enum's derived `TryFrom<u32>` conversion, whose error is
`UnsupportedFbxArrayEncoding` for every caller. Unsupported codes are rejected
before reading the payload length or payload; the supported enum remains typed
through array decoding. Blender's published binary format description documents
code 0 for raw contents and code 1 for Zlib; it reports that other values had
not
been observed. Those values are unsupported by this reader, without asserting
that Autodesk formally declares them invalid.

[Blender FBX binary format description](https://code.blender.org/2013/08/fbx-binary-file-format-specification/)

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

The selected native decoder's fixed-size input determines the element layout:
32-bit and 64-bit floats, 32-bit and 64-bit integers, or Boolean bytes. Raw data
is copied before its decoded-length check; compressed data reserves capacity
before inflation. Extra decoded bytes remain ignored, and Boolean arrays retain
every byte value. Host multiplication, overflow and allocator behavior retain
their existing policies.

The array-layout regression fixture records 282 public-parser observations.
Independent native records cover both header layouts, all five element layouts,
zero/one/two/three declarations, raw and Zlib payloads, short and extra data,
corrupt compressed input, exact float bits and Boolean bytes. Its 140 cases
using codes 7 or 4294967295 report the canonical unsupported-encoding error;
the other 142 observations retain their exact output. Large raw declarations
fit 32-bit native multiplication, while extreme compressed capacities are not
executed. No 32-bit runtime or allocator-safety verdict is claimed.

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

## Property counts

The binary FBX model-file reader admits the declared node-property count from
its header word into `FbxPropertyCount`. The unsigned wire identity stays
distinct from byte lengths and offsets through the internal property reader.
Native Vec capacity and loop bounds explicitly project it to the original host-
sized length.

Count admission remains open, including zero. No allocator limit or malformed-
file resource validation is introduced. The host-sized conversion retains the
original behavior for wide header words on a 32-bit host. A null record still
reads its name before returning without allocating its declared properties.
Array counts, offsets, byte lengths and structured errors remain separate
migration responsibilities; existing header, encoding and tag owners are
retained.

A regression fixture records 74 observations from the unchanged main public
parser. Independent native wire records cover narrow and wide headers, counts
matching or disagreeing with actual property payloads, matching nested nodes,
and null records with extreme count words and truncated names. Extreme non-null
allocation requests are deliberately not executed; no safety guarantee for them
is claimed.

## Remaining decoding audit

Binary rejection classification is owned by `FbxDecodeError`. The decoder
retains native node offsets and property-list byte lengths. Array counts and
stored payload lengths stay typed until their native buffer adapters; scalar
and element decoders use fixed-size arrays after checked reads or decoded
length checks establish the required bytes.

This does not complete framing, extent or resource admission. Native node-end
arithmetic, integer casts, allocation requests and declared offsets retain
their existing behavior. No resource limit or stricter version policy is added;
malformed offsets or excessive declarations can still panic or exhaust memory.
Array encodings retain the checked admission described above.
