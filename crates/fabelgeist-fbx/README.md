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

## Remaining decoding audit

Header-layout ownership does not complete the parser's other responsibilities.
`read_node` still carries native offsets and property counts; `read_props` and
`decode_array` still carry property tags, array counts, encoding codes and
scalar widths. Scalar conversions still contain guaranteed-length `unwrap`
calls. Those values and the parser's errors need their own complete
producer/consumer migrations. They are explicitly outside this header-layout
change; no stricter framing, encoding, version, allocation or error policy is
introduced here.