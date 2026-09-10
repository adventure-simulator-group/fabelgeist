# Independent candidate 43 code review

Bounded read-only source follow-up on 2026-09-10 to `code-42.md`, covering only
the upper-extrusion span and extraction of `boundary_edges`. No implementation
edits, test runs, or geometry generation were performed.

No concrete P1/P2 defect identified in these changes.

- Increasing `UPPER_RIM_ROWS` from two to three keeps all accesses in the
  front main grid. With 33 main rows, row 29 is the fixed reference and rows
  30, 31, and 32 receive blends of one third, two thirds, and one respectively.
  The reference is never overwritten; the skirt is excluded. Interpolated
  vectors are normalized before solidification and pass through the existing
  front refinement consistently. This widens the continuation of the gauge
  field without changing topology or morph correspondence. Source inspection
  alone does not prove that the revised outer wall clears every morph.
- `boundary_edges` preserves the previous undirected key, ordered `BTreeMap`
  traversal, and oriented single-use edge. Two-use edges still require exact
  opposite orientation; all other cases return `Degenerate`. Appending the
  returned edges in order therefore retains cut-wall vertex/index ordering,
  winding, welded aliases, and source-mid aliases for accepted inputs. The
  helper now validates every edge before appending walls; invalid inputs still
  return the same error without exposing a partially generated mesh.

The implementor reports that the previously failing tapul `corner06` now has
zero self/body contacts. That result was not rerun in this review, and the full
candidate 43 identity sweeps and final gates were still running when requested.
The candidate 42 visual regression assessment remains the last image inspected
by this reviewer; this source follow-up does not independently certify a new
candidate 43 render or all-preset acceptance.
