# Hair API contracts

`Look` accepts purpose-specific counts, physical units, a hair seed and named
grooming policies. Strand identities remain distinct from collection indexes;
parting membership represents unassigned strands explicitly. JSON keeps its
numeric and Boolean representation through serialization adapters.

The numerical operation graph receives native values in `look/groom.rs`.
Lengths use metres in the head frame; twist uses radians per metre, curl
frequency uses turns per metre, and acceleration uses metres per second squared.
Shader records and file codecs encode identities at their boundaries. Shader
layouts, hash words, random draws and floating-point evaluation order are
unchanged.

File readers return `HairDataError`, which identifies the format, malformed
field or archive member and retains parsing and I/O causes. GPU operations
return `HairGpuError`, which distinguishes invalid groom inputs, capacity
failures, readback failures and underlying device errors. Compute readback and
prefix sums own their errors and typed extents. Readback now rejects an
overflowing staging extent before allocation and propagates device polling
errors. No numerical tolerance is relaxed.
