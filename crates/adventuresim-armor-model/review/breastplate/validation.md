# Final static validation

Candidate 43, built from the implementation on `codex/breastplate-shapes-fluting`
based on `origin/main` at `e219b008`. No catalog runtime GLBs were replaced.

| Recipe | Configurations | Failures | GLB SHA-256 |
| --- | ---: | ---: | --- |
| rounded | 110 | 0 | `3504c652fc81df97d2adc132ff554105e17c7948db3452eea2d2daaf951a53ac` |
| tapul | 110 | 0 | `73234815fd4d414c30a19b476d08a47a0106e8b6c758016825f28b546662aecd` |
| peascod | 110 | 0 | `0006e28de4159f0e9e1fcd6d956ef66618aac3b2ffdcd7e39b71a26ed0f05b8b` |
| fluted | 110 | 0 | `eb539f78fa151721fd02cadce73388262056098e167a516aaae44add445558a8` |

Body: `assets/animations/biped/unarmed/base.glb`, SHA-256 `df895c53241860a1607cb7cea22546c50b497ccba319b1f5f96f78fac1740d8f`.
Builder: `target/breastplate-review/creator-43.exe`, SHA-256 `a968b2f7232c85ed1901d56e75f51c774d691ad115e655bb19a23b79e0666156`.

Each export contains the 47-channel morph layout. The 110 static configurations
exercise the 45 identity channels at signed 0.35 bounds, with aggregate and
seeded mixtures; the two skeletal channels remain zero. All tested recipes have
zero nonadjacent triangle intersections, zero body triangle contacts, and no
samples more than 1 mm inside the body. The minimum sampled clearance across
all four sweeps is 3.288 mm. Physical closure, winding, export attributes, and
morph seams pass the export audit. Full weights and diagnostics remain in
`target/breastplate-review/43/{style}/morph-checks.json`.

Additional neutral-body checks pass for the following recipes. Their inputs are
in `target/breastplate-review/42/variants`; candidate 43 outputs and reports are
in `target/breastplate-review/43/variants`.

| Variant | Count | Width / pitch | Relief | Spread |
| --- | ---: | ---: | ---: | ---: |
| two-wide | 2 | 85% | 4 mm | 65% |
| 24-narrow | 24 | 35% | 1 mm | 65% |
| 24-broad-field | 24 | 85% | 4 mm | 85% |
| ridged-fluted | 16 | 65% | 2 mm | 65% |

Verification: 47 armor tests pass (`tests-43.log`), and 7 creator design-input
tests pass (`creator-tests-42.log`). Final `fmt-43.log` and `lint-43.log` gates
pass; these logs are under `target/breastplate-review`.

This is sampled static authoring validation. It does not certify every parameter
combination, continuous body containment, skeletal animation, or installed game
assets. Gauge is preserved along the carrier extrusion vectors, not necessarily
perpendicular to the deformed or fluted surface.
