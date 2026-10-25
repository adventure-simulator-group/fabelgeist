# Complete renderer Comment Review at 2b646

Pin: `2b646df899ee73045795b129bfbade5dec7d4b84`. Tree:
`8ff9d17406c41db64dd32c901ad1bf054caf5475`. Comparison:
`162b152c47ea3824b4bbcdea01215fe5342f71f0`; pinned main
`990430b77a9f63548ffcb642074185f61ddb3322`. Canonical combined dependency
source: `e2162708fcf3885193db434300ed6c2999ab7607`.

All 18 renderer files were freshly read completely, including all three shaders
and the existing inline fixture source. This reopens the whole module; the
previous eight-file f905 review is not treated as full renderer coverage. JSON
records individual paths, blobs, SHA256, byte counts and exact coverage of
additional API/export/feature/error/math/native-buffer dependencies.

## Actionable finding

**RENDER2B-C1 — P3, `render/hair.wgsl:7–9`: stale coverage mechanism.** The
introduction attributes widened thin-strand coverage to alpha-to-coverage. The
scene pipeline passes `false` for strands and cards (`render/mod.rs:559–570` and
`517`), while `strand_fs` writes an explicit stochastic sample mask
(`hair.wgsl:408–445`). The later comment correctly explains this distinction.
Replace the introduction's mechanism wording with:

> drawn that wide and faded by the share of it that is really there, which
> a stochastic sample mask turns into multisample coverage: sub-pixel hair
> integrates to the right density instead of aliasing.

Root owns the renderer family. This requests only accurate comment wording;
shader expressions, entry points, PODs and existing fixtures stay unchanged. No
reviewer source edit was made. Exact successor review remains required.

## Contract reconciliation

The removed local atlas conversion matches canonical `HairGpuError::Atlas` with
`#[from]`, preserving its source instead of duplicating an implementation. Both
repaired imports use the actual public math `units` module; the math crate root
does not export `Ratio` or `Metres`. `render` enables the necessary `gpu`, image
and callback channel features. Canonical exports supply the retained source,
shape, simulation frame, card layout and native ports used here.

The current root alone still has older producer/error definitions, including no
canonical Atlas variant. Full agreed canonical integration is mandatory before
claiming a complete or compiled graph. The receiving contracts match the exact
canonical source read here; that does not make pending integration complete.

Two older receipt paths differ from e216 and were freshly reconciled:
GpuCards now seals its layout, device and ports behind canonical getters;
CardError adds source-shape and guide-address causes. They are not carried
forward as byte-identical source.

Source/binding identity, host/native mode, maximum resident binding prefix,
logical zero versus positive native binding floor, copy usages and concrete
error causes match their admitted contracts. Host packing keeps xyz with w=1;
native body extent comes from actual producer Buffer.size. FramePopulation
reports the selected primary resident population before stochastic and
transition cuts, with actual generic indexed triangles rather than inferred
quads. Source/device association does not prove global native health.

Deformation admission precedes parameter writes. State publication follows
normal raw submission return without promising GPU completion, rollback or an
observed native validation result. Readback documents its explicit native wait,
browser callback progress, format limits and separate SDK mapping-copy
allocation accurately. Existing prior-rest reuse, mapped-view lifetime and
logical buffer prefix wording remain consistent with their implementations.

## Source preservation and limits

The three WGSL files and five POD definitions are byte-identical to f905. The
940-byte fixture remains SHA256
`44c66d149be12de0828289a8461e243c235facb1032230263538ebc471cfda5e`: two existing
cases, one helper and five assert_eq sites. Only the stated three renderer files
differ from 162: atlas conversion removal and two imports. These are source
comparisons, not executed validation.

No source edits or project execution, build, lint, tests, scanners, generators,
runtime, push, CI or external posts. No test/case/helper/assertion/doctest was
added or run. Additional owning-source coverage is precisely bounded in JSON;
unrelated GPU/simulation module bodies are not claimed freshly fully read.
Previously source-verified SDK29 limits are retained without a new runtime
claim.

Whole canonical producer/Creator/caller and lock integration, retained shared
ownership dependencies, complete integrated thirteen-lens audit waves, three
independent final reviews and hosted checks on the same final pushed revision
remain required. One comment repair is actionable; no whole-family, compiled,
global or final clearance is awarded.
