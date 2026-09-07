# Weapon and shield mesh quality findings

Evidence links below refer to local, ignored audit output from the verification
run; those generated files are not included in the repository. The results and
reproduction commands are recorded here, and the audit regenerates its reports.

## Current construction results

The revised generator passes the complete **1,024-case sweep**, including all
**126 default/LOD cases**. All 1,024 inputs pass production validation. The audit
examined **2,894,998 source triangles**, plus float32 part geometry and actual
GLB exports. After the final fitting correction, all **105 shaped-shield cases**
were rerun; the consolidated evidence records both runs and their exact inputs. Evidence: [final report](output/mesh-quality/final-verified/report.md),
[exact cases](output/mesh-quality/final-verified/cases.jsonl), and
[generator/test source hashes](output/mesh-quality/final-verified/source-hashes.json).

| Finding | Baseline cases | Revised cases |
| --- | ---: | ---: |
| Any hard integrity failure | 256 | **0** |
| Self-intersecting shells | 178 | **0** |
| Non-manifold vertices | 76 | **0** |
| Disconnected default assemblies | 75 | **0** |
| Triangles above the advisory 100:1 aspect budget | 770 | 158 |
| Multiple shells within a part | 273 | 109 |
| Contact between shells within a part | 273 | 109 |
| Sampled thin wall | 1 | 1 |

The worst source aspect ratio fell from approximately **634,072:1 to 665:1**.
The remaining worst triangle is on the high-detail default halberd's axe plate.
There are still 17,664 triangles above the advisory 100:1 threshold; this is not
a claim that every triangle now meets that budget. The thin-wall diagnostic
still affects the targe boss. Inter-part overlaps remain inventoried, with
shield-front exclusion and default assembly connectivity enforced; arbitrary
joint-specific forbidden regions still need explicit authoring.

### Construction changes

- Resolve crossbow loft stations within ordered cavity intervals and insert
  named joints along the receiving face's inward direction.
- Triangulate actual concave blade caps, share refined cap/wall boundaries,
  and build tapered plates at their final thickness.
- Lift both shaped-shield skins from one triangulated silhouette. Construct
  pointed rims as nested perimeter bands rather than folded circular sweeps.
- Fit the whole shaped-shield fitting footprint inside its receiving region,
  including the maximum-taper/maximum-spacing interaction found by the exhaustive
  matrix. Sample the emitted shield back for fitting feet. Derive nock, vane, bridle,
  sight, bearing and inlay seats from the receiving construction.
- Build the figure-eight guard as a solid with two apertures. Preserve separately
  assembled guard members as independent closed parts. Bound tight sweep
  sections before meshing and preserve exact periodic seams.
- Subdivide long spans and narrow blade tips while retaining authored stations
  and parameter values. Snap the final sampling interval to its exact endpoint
  to avoid a nearly coincident root ring.

All independent integrity audits remain under tests; production construction
imports none of them. There are no randomized parameter changes or retry-until-
accepted generation paths. Passing this finite corpus does not prove every
combination in the continuous slider domain.

### Regression verification

All **150 ordinary test definitions pass**, including the 20 independent audit
fixtures and 10 focused construction regressions. The large preset-control
matrix exercised **34,906 combinations**. To bound wall time, 149 tests ran
with that one matrix excluded, and its unchanged body was partitioned by preset
across eight Node processes. Per-partition exact counts and the original global
coverage contract were asserted. The four partitions containing shaped shields
were rerun after the fitting correction. See the
[regression manifest](output/mesh-quality/final-verified/regression-summary.json),
[ordinary-test log](output/mesh-quality/final-other-tests-v3.log), and
[matrix rerun log](output/mesh-quality/final-matrix-affected.log). The matrix uses
production structural validation; it does not run the independent intersection
audit on all 34,906 models. The independent 1,024-case audit is reported above.

One implementation agent performed the changes and verification. No agents
were delegated; the parallel processes only ran tests.

### Runtime cost

The [interleaved benchmark](output/mesh-quality/build-performance.json) compares
five warmed builds of each of the 42 defaults at medium detail against
revision a08d3342. Total triangles across those defaults rose from 90,040 to
231,578 (**2.57x**). Summed per-preset median build times rose from about 1.61 s
to 5.46 s (**3.40x**). Other tests were running concurrently, so these timings
are an indicative comparison, not a frame-time guarantee. Shaped shields show
the largest build-time increases. Reducing construction and tessellation cost
without losing these integrity contracts remains useful follow-up work.

## Original baseline

Baseline audit of generator revision `a08d3342`, on branch
`codex/weapon-mesh-quality`, 2026-09-07. The branch now also contains
constructive generator fixes; the results below describe the original baseline.
The authoritative baseline evidence is in
[`output/mesh-quality/verified/`](output/mesh-quality/verified/report.md).

The verified sweep examined **1,024 cases and 1,257,624 source triangles**.
**256 cases failed**, including **87 of the 126 default/LOD cases**. All 1,024
inputs passed the current production validator. The failing categories overlap:

| Integrity failure | Cases |
| --- | ---: |
| Self-intersecting individual shells | 178 |
| Non-manifold vertices | 76 |
| Disconnected default assemblies under the strict contact rule | 75 |

The ordinary `npm test` run passed **135 tests** (including the audit fixtures
present when that run started). The latest standalone audit-fixture run passed
**20 tests**. The baseline corpus acceptance test exited with failure on the
generator defects described below.
Evidence logs are `output/mesh-quality/baseline.log`, `fixtures.log`, and
`verified.log`. One implementation agent performed the work.

No additional failures were reported for buffer/index validity, degenerate or
duplicate triangles, edge closure/winding, shell orientation, float32 conversion,
the tested GLB round trips, LOD shell/Euler signatures, or shield front/fitting
intersections. That is a result for this corpus, not a guarantee for all inputs.

## Confirmed deficiencies

| Deficiency | Small reproduction | Evidence and likely repair area |
| --- | --- | --- |
| Fullered blade end caps overlap their own side surfaces | Default Landsknecht longsword, low LOD; no control changes needed | `minimal-blade.json`. `sectionBlade` fans caps from the origin even though the fullered cross-section is not star-shaped about that point. Triangulate the actual section polygon. |
| Guard pieces meet at non-manifold vertices | Landsknecht longsword: set **Quillon terminals** to **scroll** | `minimal-guard.json`. The vertex link has disconnected fans. Compound hilt and figure-eight constructions also produce these failures. Decide which pieces are independent assembled solids and which require a continuous joined surface. |
| Crossbow rear stock folds through itself | Composite arbalest: **Tiller length = 0.92 m**, **Draw length = 0.30 m** | `minimal-crossbow.json`. The source triangle intersection survives a stricter 1-picometre predicate tolerance. The ordered stock loft stations need a geometric ordering invariant. |
| Shield body and rim can intersect themselves | Heater shield: **Bottom depth = 0.30 m**, **Side taper = 0.65** | `minimal-shield.json`. This two-control input remains valid under production validation and fails the independent geometry audit. Check the planar strip layout and swept rim at shaped ends. Other sweep cases affect kite and Roman tower shields. |
| Polearm butt caps float below the shaft | Default halberd, medium LOD | `gap-halberd.json`: nearest triangle surfaces are **5 mm** apart. The shaft begins at Y=0; the cap ends at Y=-0.005. Attachment-frame agreement currently accepts this. |
| Shield handles float behind the body | Default heater shield, medium LOD | `gap-shield.json`: closest body/handle surfaces are **3.05367 mm** apart. The other shield defaults also fail the contact graph. |
| Extremely slender triangles | Glaive at combined maxima, low LOD | Longest-edge/altitude ratio reaches **634,072:1**. Default Dussack at high LOD reaches **365,200:1**. These are triangulation diagnostics, independent of weapon practicality. |

All minimal definitions above are stored under `output/mesh-quality/verified/`.
The minimizer preserves production validity and the same part/finding while
resetting controls to defaults. The reductions are 1-minimal, not proofs of a
globally smallest reproduction.

## Findings that need a policy decision

The default contact graph uses a strict **2 micrometre** distance budget. Some
failures are obvious detached models, as measured above. Others are clearances
in an otherwise retained assembly: the default self-bow string assembly is
**0.108454 mm** from its nearest nock overlay (`gap-bow.json`). Such findings
should not automatically be treated as equally severe or as proof that the
weapon could not be assembled.

Inter-part overlaps and contacts between multiple shells inside a named part
are inventoried without a blanket rejection. Fitted sleeves, tangs, and
decorative furniture need explicit joint regions. The audit supports and
fixture-tests required contact, forbidden contact, clearance, containment, and
convex joint-envelope checks, but most generator joints still lack authored
regions. This is a remaining coverage gap, not a clean clipping verdict.

Aspect ratios above 100:1 are review findings. Thin-wall sampling likewise
reports rather than rejects: the high-LOD targe boss has a sampled normal
thickness of about **46.49 micrometres**. The sample is not a global minimum
thickness guarantee, and a cutting edge would need different treatment.

## Tests and reproduction

`npm run test:quality` runs the broken/valid geometry fixtures and the generator
corpus. The corpus writes its full report before exiting nonzero when any
integrity contract fails. It covers 42 presets, all three default LODs, slider
extremes and adjacent steps, choices, seeded combinations, targeted three-way
interactions, adversarial definitions, and composed heads/hafts. Non-default
cases use low LOD. `QUALITY_PROFILE=deep` adds individual slider endpoints and
adjacent steps; it was not part of this sweep.

To replay the reduced shield case in PowerShell:

```powershell
$env:QUALITY_REPLAY = 'output/mesh-quality/verified/minimal-shield.json'
$env:QUALITY_OUTPUT = 'output/mesh-quality/replay-shield'
npm run test:quality
```

See `README.md` for profile/filter controls and the minimization and gap tools.
`cases.jsonl` records every exact definition and bounded triangle/vertex
witnesses; `summary.json` retains full finding counts. Counts refer to test
cases and primitive pairs, not independent root causes.

The predicates are tolerance-bounded floating-point calculations, not an
exact-arithmetic proof. Fixture regressions explicitly cover nearly coplanar
shared vertices and periodic seams after float32 conversion so numerical
contact artifacts are not counted as real defects. Runtime viewer validation
has not been replaced by these tests.
