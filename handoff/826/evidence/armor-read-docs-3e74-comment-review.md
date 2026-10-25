# Armor public-read documentation Comment Review

No actionable comment finding in the four public Rustdoc blocks at
`3e74eea62f4138fb95e001bf93e9438966d12612`
(tree `a83c132e94bab1db7691468d553465081b7432fb`), compared with
`d7da77bb9d21200170b522b8e6a1f7aa4aef2bab`.

Both owning files were read fully with the complete diff. The only changed
paths are `gpu/facade.rs` and `gpu/staging.rs`; their non-Rustdoc lines remain
identical to the parent, and their hashes match the working proof. The JSON
records exact source identities and full, bounded and carried lower coverage.

Direct reads accurately require the facade's device and its paired queue,
producing submission handling, and the mapping obligations already owned by
`Buffer::read`. Grouped reads accurately require sources available for their
GPU copies and describe mapping the staging allocation. They introduce no
blanket exclusive grouped-read or alias guarantee, device-provenance check,
completion of later work, retry, rollback or global device-health promise.

Root still calls the unreceived canonical grouped Readback API. Its actual
old lower API and stale always-stage/CPU-never-stall prose require owner780
integration and review. This documentation clarification does not resolve that
mismatch. Remaining producer/caller/cache/lock migrations, all thirteen lenses,
final independent reviews and hosted gates remain required.

No source edits or project checks, builds, tests, scanners, generators, runtime,
push or CI were performed. No cases, helpers or assertions were added. No
compiled, runtime, source-complete, global or final whole-PR clearance.
