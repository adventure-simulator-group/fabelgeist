# Binary FBX fixtures

These small authored files exercise the public decoder at its native
little-endian serialization boundary. They contain no third-party assets.

The version fixtures use narrow and wide node headers, ignored header fields
and property lengths, lossy node names, exact floating-point bit patterns,
noncanonical Boolean bytes, raw arrays and repeated Zlib arrays using supported
encoding code 1. They also include duplicate objects, ordered connections and
trailing bytes. Their accepted output is identical across these layouts.

The compressed short-buffer and invalid-Zlib fixtures also use encoding 1 so
they reach their intended length-check or native inflation failure. Unsupported
encoding words are covered by the separate existing array-encoding regression;
these fixtures exercise payload and layout behavior after encoding admission.

The remaining fixtures exercise ASCII/header rejection, truncated reads,
short decoded arrays, invalid zlib input, unknown tags and first-failure
precedence. A named null node and a partial trailing header remain accepted.
