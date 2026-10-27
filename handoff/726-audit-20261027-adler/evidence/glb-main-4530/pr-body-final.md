GLB exports previously calculated padded chunk and container lengths with
unchecked host arithmetic and narrowing casts. A large payload could produce
incorrect unsigned 32-bit header lengths. The writer also used an infallible
reservation for the final frame and contextual errors rather than a dedicated
GLB error type.

The writer now checks the actual serialized JSON and binary lengths before
reserving and assembling the complete frame. Chunk order, four-byte alignment,
padding, empty BIN chunks and the unsigned 32-bit format range are preserved.
Serialization, layout, final-frame reservation, directory creation and write
failures retain their concrete causes, with paths on filesystem errors. The
public export entry point retains its existing result type.

Optional timing serialization drops an unencodable diagnostic rather than
panicking after the wrapped operation. The owning README documents that earlier
export steps may publish shared images and filesystem writes can leave partial
output. The growth baseline is tightened for the shorter export file and
function; no limit is increased.

## Validation

- The [full hosted Rust quality workflow](https://github.com/adventure-simulator-group/fabelgeist/actions/runs/40339807509)
  passed on `2ef288d1085c249fa056156182ea626a060bb8d1`, including Linux and
  Windows. Uploaded evidence confirms all 48 required contract groups executed
  with positive pass counts and zero failures, the 11 named existing export
  cases passed, and the Windows portability group passed its 10 required cases.
- Independent source, rules and documentation reviews found no remaining
  findings within this change's scope.
- No tests, cases or assertions were added. Oversized frames, allocation
  failures, 32-bit execution and filesystem failure paths have source review
  only. No local project checks were run.

## Implementation context

The layout, error definitions and packet assembly have separate responsibilities.
One constructor derives all lengths from the real payloads; private fields keep
the admitted layout coherent until encoding. Arithmetic is checked before
conversion to unsigned 32-bit fields. One fallible reservation covers the final
assembled frame; earlier serialization and image allocations retain their
existing behavior. Geometry, texture generation and equipment assembly are
unchanged.

The final hosted baseline matches both lowered limits. Independent reviews cover
the source changes and the baseline-only follow-up; the evidence above applies
to the exact final commit. No reviewed public conversation snapshot is available.
