# Binary FBX array cardinality and layout

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

Property counts, property tags, array encoding, version/header layout, byte
framing and structured errors remain separate migration responsibilities.
Integrating their pending increments should retain each final owner and combine
the owning guides.
