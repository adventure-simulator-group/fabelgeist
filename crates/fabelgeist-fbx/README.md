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
original float bit patterns and Boolean byte values remain unchanged. Declared
array counts and element layouts use their own owners. Payload byte
lengths, offsets and structured decode errors remain separate migration
responsibilities.

A regression fixture records 95 observations from the unchanged main public
parser. Independently encoded wire properties cover all five array kinds with
zero, one and other nonzero codes; empty, ordinary, short and extra payloads;
and corrupt compressed streams. The fixture retains exact floating-point bit
patterns and Boolean bytes when comparing decoded output.

The native `Reader::u32` call reads the serialized encoding word, not a domain
quantity: FBX array metadata stores element count, encoding and compressed byte
length as three little-endian 32-bit words. `FbxArrayEncodingCode` derives its
transparent conversion from that decoded word and remains typed through the
array decoder. Blender's public reader uses `<III` for this same wire header.
That reference establishes field width and order; this reader deliberately
retains its existing zero/raw and every-nonzero/Zlib policy.

[Blender FBX reader, read_array_params](https://github.com/blender/blender-addons/blob/main/io_scene_fbx/parse_fbx.py)

## Property tags

The binary FBX model-file reader admits each one-byte property tag into a
bespoke FbxPropertyTag before dispatch. The same owner names all thirteen
supported tag codes and distinguishes scalar numbers and Booleans, numeric and
Boolean arrays, strings, and raw byte payloads. Tag identity remains typed
through both property selection and array selection.

Tag admission is open. Unknown native bytes reach the existing dispatch
rejection and retain the same character diagnostic. The type does not admit
decoded payloads or change scalar widths, array decoding, Boolean truth rules,
string encoding, read ordering or parser errors. Existing version/header,
encoding and element-layout owners remain distinct.
Offsets and structured errors remain separate migration responsibilities.

A regression fixture records 284 observations from the unchanged main public
parser. Independently encoded properties cover all 256 native tag bytes, all
thirteen known payload representations, missing and short payloads, and mixed
forward/reverse property sequences. Comparisons retain exact floating-point bit
patterns, invalid string bytes and the distinct string/raw property variants.

## Property counts

The binary FBX model-file reader admits the declared node-property count from
its header word into FbxPropertyCount. The unsigned wire identity stays distinct
from byte lengths and offsets through the internal property reader. Native Vec
capacity and loop bounds explicitly project it to the original host-sized
length.

Count admission remains open, including zero. No allocator limit or
malformed-file resource validation is introduced. The host-sized conversion
retains the original behavior for wide header words on a 32-bit host. A null
record still reads its name before returning without allocating its declared
properties. Array layouts use their own owners. Offsets, byte lengths and
structured errors
remain separate migration responsibilities; existing header, encoding and tag
owners are
retained.

A regression fixture records 74 observations from the unchanged main public
parser. Independent native wire records cover narrow and wide headers, counts
matching or disagreeing with actual property payloads, matching nested nodes,
and null records with extreme count words and truncated names. Extreme non-null
allocation requests are deliberately not executed; no safety guarantee for them
is claimed.

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

A regression fixture records 282 observations from the unchanged main public
parser. Independent native records cover both header layouts, all five element
layouts, zero/one/two/three declarations, raw and nonzero compression codes,
short and extra payloads, corrupt compressed input, exact float bits and Boolean
bytes. Large raw declarations fit even 32-bit native multiplication; extreme
compressed capacities are not executed. No 32-bit runtime or allocator-safety
verdict is claimed.

Existing version/header, property-count, property-tag and compression owners
remain distinct. Payload byte lengths, node offsets, scalar widths and
structured errors remain separate migration responsibilities.

## Remaining decoding audit

These bespoke types do not complete the parser's other responsibilities.
`read_node` still carries native offsets and property-byte lengths; `read_props`
still carries payload byte lengths. Native array count/layout
projections remain explicit buffer adapters. Scalar copy widths remain native.
Scalar
conversions still contain guaranteed-length `unwrap`
calls. Those values and the parser's errors need their own complete
producer/consumer migrations. They are explicitly outside these header,
array-encoding, property-tag, property-count and array-layout
changes; no stricter framing, encoding, version, allocation or error policy is
introduced here.
