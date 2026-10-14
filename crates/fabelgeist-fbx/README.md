# Binary FBX property counts

The binary FBX model-file reader admits the declared node-property count from
its header word into FbxPropertyCount. The unsigned wire identity stays distinct
from byte lengths and offsets through the internal property reader. Native Vec
capacity and loop bounds explicitly project it to the original host-sized
length.

Count admission remains open, including zero. No allocator limit or
malformed-file resource validation is introduced. The host-sized conversion
retains the original behavior for wide header words on a 32-bit host. A null
record still reads its name before returning without allocating its declared
properties. Array counts, header/version layout, byte framing, property tags and
structured errors remain separate migration responsibilities.

A regression fixture records 74 observations from the unchanged main public
parser. Independent native wire records cover narrow and wide headers, counts
matching or disagreeing with actual property payloads, matching nested nodes,
and null records with extreme count words and truncated names. Extreme non-null
allocation requests are deliberately not executed; no safety guarantee for them
is claimed.
