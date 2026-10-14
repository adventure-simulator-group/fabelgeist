# Binary FBX fixtures

These small authored files exercise the public decoder at its native
little-endian serialization boundary. They contain no third-party assets.

The version fixtures use narrow and wide node headers, ignored header fields
and property lengths, lossy node names, exact floating-point bit patterns,
noncanonical Boolean bytes, raw arrays and zlib arrays with three nonzero
encoding words. They also include duplicate objects, ordered connections and
trailing bytes. Their accepted output is identical across these layouts.

The remaining fixtures exercise ASCII/header rejection, truncated reads,
short decoded arrays, invalid zlib input, unknown tags and first-failure
precedence. A named null node and a partial trailing header remain accepted.
