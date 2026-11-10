# Ordered-sigil puzzle-lab renderer callers

Selected revision `ff9c1d7d0ea168b7e14eedc5e44879ee7b84fb04`.
Registrations: `REGISTRATION-ERRANTRY-LAB-CALLERS-L4-3821.json` and
`REGISTRATION-ERRANTRY-GENERATOR-L4-3821.json`, together with the prior
ordered-sigil generator and replay registrations at checkpoints 3818 and
3820.

The selected binary's `Show`, `Analyze`, and `Play` paths call the renderer
with a projection from `generation.generate`. That calls
`PuzzleAuthority::generate_request`; the ordered-sigil arm calls
`OrderedSigilPuzzle::generate_with_spec`, which validates its spec and the
constructed puzzle. `Replay` separately deserializes and validates the
private authority before rendering. These cover the four renderer call sites
found in this binary; the `play` call is reached with a generated authority.

This is source-derived protection for the selected binary paths, not a
runtime test or a guarantee for callers of public standalone clue APIs. The
existing clue-position contract and global alias reconciliation remain open.
