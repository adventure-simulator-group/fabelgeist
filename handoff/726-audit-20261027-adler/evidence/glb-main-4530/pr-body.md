GLB exports previously calculated padded chunk and container lengths with
unchecked host arithmetic and narrowing casts. A large payload could therefore
produce incorrect unsigned 32-bit header lengths. The writer also used an
infallible reservation for the final frame and reported write failures through
contextual errors rather than a dedicated GLB error type.

This change admits the actual serialized JSON and binary lengths through one
private checked layout before reserving and assembling the complete frame.
JSON and BIN chunks retain their order, four-byte alignment and existing padding;
an empty BIN chunk remains valid. The format keeps its unsigned 32-bit range.
Serialization, layout, final-frame reservation, directory creation and write
errors retain their concrete causes, with paths on filesystem failures. The
public export entry point continues returning its existing result type.

Optional timing serialization now drops an unencodable diagnostic rather than
panicking after the wrapped operation. The owning README documents preparation
and publication behavior: earlier export steps may already publish shared images,
and a later filesystem write can still leave partial output.

## Validation

- Hosted Rust quality workflow pending on the exact PR head. It includes the
  existing Creator export tests, formatting, semantic and growth checks,
  generated-binding checks, Clippy, and the remaining Linux and Windows gates.
- Independent source, rule and documentation reviews are in progress.
- No new tests, cases or assertions were added. Existing normal export tests
  will exercise successful output; oversized frames, allocation failures,
  32-bit execution and filesystem failure paths have source review only.
- No local builds, tests, linters, scanners or generators were run.

## Implementation context

The layout, error definitions and packet assembly have separate responsibilities.
One constructor derives all lengths from the real payloads; private fields keep
the admitted layout coherent until encoding. Native arithmetic is checked before
conversion to the format's unsigned 32-bit fields. One fallible reservation
covers the final assembled frame; earlier serialization and image allocations
retain their existing behavior.

The change starts from main after #986 and contains only the recovered six-file
GLB framing slice. It changes no geometry, texture generation or equipment
assembly. Independent source review and exact-head hosted evidence will be
summarized here before requesting merge review. No reviewed public conversation
snapshot is available.
