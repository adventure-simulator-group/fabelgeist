# Corrected boot25 runtime evidence

Four bounded captures completed after rebuilding the viewer against the installed accepted boot25 pair. All prior captures and provenance are preserved; this checkpoint supersedes their boot-specific evidence. No Rust or animation changes were made for this recapture.

| Capture | Frames | PNGs | Exit | Failed validation | Jitter incidents |
|---|---:|---:|---:|---|---:|
| mail-boot25-final-idle | 65 | 195 | 1 | None | 7 |
| mail-boot25-final-guard | 55 | 165 | 0 | None | 0 |
| padded-boot25-final-idle | 65 | 195 | 1 | None | 11 |
| padded-boot25-final-guard | 55 | 165 | 1 | biomechanics_within_review_bounds | 0 |

All **720 PNGs** and all planned scenarios completed. Every capture passed the actual gameplay armor mesh/material/morph/wearer-skin readiness gate. No GPU layout errors occurred. All captures pass ground-penetration validation.

Mail guard passes all viewer validation. Idle exits reflect startup jitter already reproduced with an unarmored control. Padded guard fails only the biomechanics foot-dragging category, with zero jitter; it is not reported as a full animation pass. That category was also flagged in the preserved earlier `plate-final-guard` run, before the boot25 correction; no threshold was relaxed and no extra capture was run to seek a passing result.

All 16 distinct fixture GLBs were rehashed and match the per-run provenance. Both boots match the accepted installed pair in all four captures:

- Left: `bab454252766b20c27b28d5c5c2846da5d0c8f0375bffa0d764200e8896473d2`
- Right: `55041e3bfd6d0d51d80f19efa0a1d5bf583c9e2a1828ba4fff5cf66075f1f9f6`

Executable SHA256: `52866C8A6A6C4B57FE369307D44F56CC72D4F2ECC561A924DE25EF865BDD9EA8`.
Installed manifest SHA256: `6AEF2697710E03698154B8B5E5F56804ED3D8A09D5C7C9A95B8B09272DB9B302`.

Raw evidence: each named capture folder contains `manifest.json`, `runtime-provenance.json`, `armor-fixture.json`, body proportions, bone trace and original PNGs. Adjacent stdout/stderr and `boot25-capture-status.txt` preserve execution results. `boot25-runtime-summary.json` contains exact checks and hashes. Viewer build passed: `../occupancy/boot25-viewer-build.log` and `../occupancy/status.txt`.

Independent reviewer `final_body_regression` was notified on the first complete mail idle run, then mail guard and both padded runs. Visual acceptance remains that reviewer’s responsibility. No capture/build process remains active.
