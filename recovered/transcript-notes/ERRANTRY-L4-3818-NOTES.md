# Lens 4 errantry clue admission and projection trace

Selected revision `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.
Registrations: `REGISTRATION-ERRANTRY-CLUE-L4-3818.json`,
`REGISTRATION-ERRANTRY-PROJECTION-L4-3818.json`, and
`REGISTRATION-ERRANTRY-AUTHORITY-L4-3818.json`.

- `OrderedSigilClue` has public `u8` positions and derives deserialization;
  `text()` adds one directly, and `fey_clue_text` uses the raw position as an
  index. `OrderedSigilPuzzle::validate` rejects out-of-range coordinates, but
  it is an explicit aggregate method, not construction or deserialization
  enforcement on the standalone clue or observer projection.
- The inspected backend challenge view deserializes private `PuzzleAuthority`,
  invokes its `validate` dispatch, and only then serializes a projection. The
  ordered-sigil dispatch calls `OrderedSigilPuzzle::validate`; therefore the
  inspected successful backend projection has this coordinate check. Invalid
  authority rows are filtered out by the view, rather than projected.
- The inspected web route deserializes the projection JSON and passes it to a
  template that calls `fey_clue_text` per clue, without an additional shown
  coordinate check. A malformed projection from another source or direct use
  of the public API could differ from the validated backend path; no actual
  unvalidated production producer or runtime failure is established here.
- This refines retained `ROOT-INITIAL04-F049` and is not a new unique
  finding. Other producers, error visibility, native acceptance and exact
  cross-lens aliases remain open. No product execution or repair occurred.
