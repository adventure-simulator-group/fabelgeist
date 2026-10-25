# Sole shell UV authority: independent Code Review

Reviewed exact peer `d000e848f2f5bc28b0728c2cd664c727a41ecf50` against
`59bb96d07af08244582c6767d81e969b820ec3ed`. Root has not copied this packet.

No actionable correctness finding in the six-file delta and the traced UV
consumer closure. All six manifest, export and Git hashes match independently.
All six complete source files and their complete diffs were freshly read,
including the existing fixtures in the 1,785-line export file. The JSON report
records exact per-file hashes and explicitly scoped dependency coverage.

`RiggedShell.surface` now stores maps only. `texcoords` is the sole vertex UV
column. Validation requires finite, position-sized UVs for either map family
before compaction or output. Untextured shells can still omit the column.

The unchanged compactor applies the same ordered used-vertex selection to UVs,
geometry, skin and morphs. Its returned shell borrows compact UVs, while its
surface retains only the map payload. The writer reads only that compact UV
column, so it cannot recover stale source-size UVs through a fallback.

Actual armor producers already set `texcoords`; both catalog export paths now
attach only maps. Trim selects its own column explicitly. Unchanged mail, hair,
textile and untextured clothing paths fit the final API. Material map ordering,
image identity caching, samplers, PBR factors, image packaging and numerical
serialization are unchanged. The existing textured fixture moves its same UV
slice to `texcoords`; no test, case, helper or assertion was added. Exact text
bookkeeping confirms the whole export file differs only by those two fixture
field operands.

Receive all six mutually dependent files through the agreed ownership and
composition process. This review does not clear the wider material, appearance,
recipe, units, count, native cause, bake, body, physical or generated graph.
Final reviews and hosted checks on the same pushed revision remain required.

No production source was edited or copied. No project build, check, lint, test,
scanner, generator, runtime, push or CI was run. This is bounded source evidence,
not compilation, runtime, whole-PR or final clearance.
