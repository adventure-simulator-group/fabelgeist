# Ordered-sigil additional consumers

Selected revision `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.
Registrations: `REGISTRATION-ERRANTRY-CONSUMERS-L4-3820.json` and
`REGISTRATION-ERRANTRY-LAB-L4-3820.json`.

- The puzzle-lab ordered-sigil renderer calls `clue.text()` directly. The
  inspected replay command deserializes a private puzzle authority, validates
  it, checks deterministic replay, then renders its projection. This protects
  the selected replay path; other renderer callers need separate assessment.
- The other inspected `clue.text()` in the web challenge template is for
  logic-grid clues, not ordered-sigil coordinates. It must not be counted as a
  second consumer of this specific position contract.
- The web submit path also deserializes the challenge projection and builds a
  submission; it is distinct from the GET rendering path that calls
  `fey_clue_text`. No malformed live source, product execution, new unique
  finding, or contract closure is claimed.
