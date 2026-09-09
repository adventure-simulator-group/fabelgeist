# Boot25 installed: final boot layering and ankle flow

Boot25 is installed in both `assets/equipment/procedural/` and `target/armor-review/final-catalog-stage/`. Only the two leather-boot GLBs and their manifest rows were promoted; all 49 other rows were verified unchanged. The runtime review coordinator was notified immediately after installation. No commit was made by this implementor.

- Left SHA-256: `bab454252766b20c27b28d5c5c2846da5d0c8f0375bffa0d764200e8896473d2`.
- Right SHA-256: `55041e3bfd6d0d51d80f19efa0a1d5bf583c9e2a1828ba4fff5cf66075f1f9f6`.

## Final construction and review

`boot_layer_fit.rs` owns the independent dressed-shaft constraint and conservative outward fairing of ankle meridians. Actual fitted mail and padded chausses triangles supply complete plane sections. The existing anatomical boot is fitted first; a dressed contour expands points that need it, then fixed lower-foot/upper-shaft anchors and original radii constrain outward fairing through the ankle valley. This produces continuous leather flow rather than a projecting circumferential sleeve shelf. `limb_fit.rs` only invokes the focused fitter in the leather-boot branch. Garments, other limb pieces, and the frozen gorget/rerebrace corrections are untouched.

Boot23's earlier numeric pass did not establish artistic acceptance: the fresh critic rejected its lower-shaft shelf and the root withdrew its earlier fold interpretation. Boot24 contour-only diagnostics retained the valley and were not exported. Boot25's outward meridian fairing received broad critic PASS, moderate confidence, after actual inspection of the historical reference and current front/side/quarter images (`reviews/boot25-broad.md`). The localized side fold remains a stated minor reservation. Root independently inspected front/side and accepted the scoped simple working-boot shape. Independent source review reported no P1/P2 findings (`reviews/boot25-code.md`).

The maximum toe-forward extent and sole-bottom plane remain exactly unchanged. Distal rounded toe vertices at world z > 0.130 m move at most 0.44 mm from wall-normal regeneration; the foremost vertices at z > 0.150 m do not move. Some dorsal forefoot/instep points near y = 0.041 m, z = 0.100 m move up to 19.23 mm as the fair bridge fills the ankle valley. Sole-region vertices move at most 0.276 mm. Root reviewed these localized preservation measurements and authorized final export. The sole index arrays and all connectivity remain unchanged.

## Exact final artifacts

`boot25-stage/` is the ordinary creator's filtered two-boot export with all 47 morph targets. `boot25-checks/provenance-and-parity.json` records final source hashes and both GLB hashes. Neutral positions match the critic-reviewed source helper exactly on both sides (maximum difference 0); helper and GLB index arrays are identical. Actual source helper: `boot25-helper/`.

Actual GLB colored views for all four requested bodies and both garments are in `boot25-interfaces/{neutral,positive,negative,mixed}-{mail_chausses,padded_chausses}/`. Each has combined front/side/quarter views plus isolated component diagnostics. Orange is boot, cyan is the selected leg garment, magenta is ipsilateral skin. The opposite leg is omitted to avoid the prior side-view ambiguity; positions are never modified for rendering. Critic-reviewed helper views are in `boot25-helper-interface/`; padded helper views are in `boot25-helper-padded-interface/`.

## Checks

- Both sides have zero triangle crossings with both mail and padded chausses in all 51 reference-space configurations: neutral, all 47 exported endpoints, and positive/negative/mixed identity blends at magnitude 0.35. Evidence: `boot25-checks/triangle-intersections.json`.
- Both boots pass `check_armor_review.py` for all four body states in `boot25-morphs/`: zero vertices, face centroids, and unique edge midpoints more than 1 mm inside. Minimum vertex clearance is +5.727 mm; worst sampled interior signed distance is −0.0088 mm, well within the established threshold. Physical edge closure/winding passes.
- Exact body triangle contacts in negative/mixed blends remain confined to the unchanged sole fan. Old/new contact details are in `boot25-checks/body-contact-detail.json`, using the backed-up previously installed boots as the baseline. This is distinct from the new zero-crossing boot/garment contract.
- Both GLBs pass `check_parametric_armor_assets.py --allow-partial`, covering 47 target correspondence, blends, physical seams and triangle areas. Log: `limbs/boot25-glb-audit.log`.
- `just fmt-check`, full `just lint`, and all 44 creator package tests pass; one existing asset-dependent test is ignored. New regression tests cover complete sections between tilted rings and outward valley filling with fixed anchors. Logs: `limbs/boot25-tests.log`, `limbs/boot25-lint.log`, `limbs/boot25-clippy.log`.

Installation evidence and untouched-row assertions: `boot25-checks/installation.json`. Previous boots/manifests are retained under `boot25-install-backup/`. Runtime animation acceptance is the remaining coordinating review step; static reference-space checks do not claim animated clearance.
