# Independent material PBR pre-receive Comment Review

Revision: `ead46619dec5ef45993f63202cb289bd3986b018`  
Tree: `d5cd6f362a0044c4df16c00d6960a88e23b2b421`  
Compared with `d000e848f2f5bc28b0728c2cd664c727a41ecf50`; Root receiving
anchor `ccb1b6944d674c0bf29d10e6820c0ac4c55092b7`.

The frozen packet has two connected documentation findings. This is a
pre-receive source review, not whole-family or final clearance.

## Findings

- **MATERIAL-PBR-EAD-C1, P3 — obsolete uniform ring rationale.**
  `crates/adventuresim-character-creator/src/device_fit.rs:145–146` says a
  kernel's uniform ring must not wrap within a batch. Exact-pin
  `kernel/fast/arena.rs:41–83` appends and retains a new chunk when full;
  `KernelBatch` owns that arena. Preserve the existing one-submission-per-shape
  behavior, but replace this rationale with “Record and submit each
  realization separately.” or remove it. Do not invent a performance claim.
- **MATERIAL-PBR-EAD-C2, P3 — explain the white factor.**
  `crates/adventuresim-character-creator/src/character_export/hair.rs:111–112`
  says glTF takes a texture as the whole base color. The exporter instead
  explicitly uses a white scalar factor when a color map is present:
  `export/textures.rs:74–82`, then `export/shell.rs:55–72`. Suggested wording:
  “The largest atlas level tinted in sRGB, with coverage as alpha. The hair
  shell uses a white scalar factor so the texture supplies its color and
  opacity.” Preserve tinting, coverage and packing behavior.

Both items belong to the connected Creator receiving documentation and require
coordination with its owner. No source was edited.

## Coverage and verified contracts

All **75 non-lock selected files, 16,588 lines**, were freshly read in full,
including their existing fixture bodies and documentation. All 77 packet hashes
were independently compared with exact Git source. The JSON records each file,
hash, extent and disposition. Peer notes and historical verdicts gave no current
source-read credit.

The two generated locks account for 25,437 lines. Their complete two-line
comment sets, complete textual base-to-pin changes, and relevant Bevy 0.19.1,
egui 0.34.3 and color package records were read. Remaining unchanged dependency
records were identity-accounted, not freshly semantically resolved. This is an
explicit bounded lock review, not a claim to have read all 42,025 lines in full.

The encoded leaves explicitly leave numerical admission to recipes; RGB decode
may overflow, and alpha is copied directly. The terminal linear adapter checks
its decoded finite outputs without claiming complete geometry, range or GLB
admission. Bevy uses its encoded `Color::srgba` API. Local pinned Bevy and egui
SDK excerpts confirm the encoding and float-picker claims.

`ComponentAppearance` owns the single mandatory Inherit/Surface/Textile payload;
textile scalar appearance derives from the current mutable textile payload.
Default, gilt, trim and native producer adapters use that authority. Color maps
select white scalar color/alpha before decoding; normal-only maps retain the
authored appearance. Shared body skin is separate from scalp shading.

Both review JSON producers use the same checked `ReviewComponent` projection;
the unchanged renderer consumes its material/range shape. Its documentation
correctly limits checking to scalar factors. Existing caches retain their first
initialization outcome, including failure. Ordinary submission, readback,
provider handling and host-float caveats do not promise global health or
transactional publication. C1 removes a stale reason without redesigning batches.

## Required limits and dependencies

No source edits, local project execution, checks, builds, tests, scanners,
generators, runtime rendering, push or CI occurred. Existing fixture changes
inspected against `d000` adapt fields, types and operands; this receipt does not
prove every original fixture across the whole PR or execute those fixtures.

Physical material units/count/seed/closed Engraving/trim/application causes;
canonical core/native/generated/stride/caller/lock composition; and the retained
#774 fitting/body/SurfaceCut/cache families remain required. The Open versus
cyclic textile seam-policy relation stays in physical-family discovery; it is
not presented as a completed physical audit or a separately proved comment bug.

UV-R1 export admission/body finite/native assembly, UV-R2 checked GLB framing and
allocation, and DENS-R1 typed missing design remain open. Historical compute
numerical failures and Creator SIGSEGV are historical limits, not a new packet
verdict. Complete all thirteen lenses, all three final reviews and required
hosted checks must still pass on the same final pushed revision.
