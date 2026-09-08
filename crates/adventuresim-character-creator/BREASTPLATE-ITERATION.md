# Reproducible breastplate renders

Pass `--breastplate-design <file.json>` to load a complete `BreastplateDesign`.
The same input applies to the interactive studio, `--export-only`, and
`--generate-equipment`. Omitting it preserves existing defaults. Invalid JSON,
missing/unknown/duplicate fields, and out-of-range values fail before loading
body assets.

Start with `breastplate-design.example.json` in this directory and change one
field at a time. The millimetre fields are `crown`, `skirt_flare`, `clearance`,
and `wall_thickness`; other numeric fields use permille. These are existing
generator inputs, not a guarantee that the fitted output moves by exactly the
entered amount. Body support and surface fitting constrain the result.

From the repository root, add the following to an existing creator command:

```text
--breastplate-design crates/adventuresim-character-creator/breastplate-design.example.json
```

The example matches the current default. Keep the wearer recipe, design JSON,
exported mesh, and body-included render together when comparing iterations.
Use a separate output directory for each equipment export.

## Experimental angular surface

Latest private continuation: native35A12/default/round-00 FAIL. Global cubic
radius profiles in35A11 improved longitudinal surface flow, but independent
testing found a lower-rim clearance miss and poorly seated upper strips.
35A12 adds an explicit11.5–16.5mm upper-rim distance envelope. Its saved finite
mesh fixes that rim regression, but110/130 upper-edge samples exceed16.5mm and
the optimizer stops after4 iterations. Broad reference review also fails the
armhole-to-side handoff. Independent saved-Jacobian analysis finds a local
constraint conflict even without determinant coefficient rows; this does not
prove the nonlinear family impossible. The next bounded experiment will assess
more independent chest/shoulder shape authority. No new source implementation
has been promoted: Rust remains35T, all-morph/thickness acceptance is outstanding.
Private breastplate-review/RESUME.md owns the current continuation. Historical
entries below are preserved rather than retroactively relabeled as passes.

Latest private continuation: native35A10/default/round-02 passes the declared
finite default-body clearance checks (11.6358mm actual triangle minimum against
11.5mm), with independently checked current contact derivatives, disk topology
and shared smooth surface. The broad reference critic and root arbiter still
FAIL its upper-band/main-wall shape progression and armhole organization. This
is not a production, continuous-patch, solid-shell or morph-family acceptance.
Source remains35T. Private RESUME.md in the breastplate-review directory owns
current experiment state. The next investigation is longitudinal shape authority,
not more fitting against stale contact planes;35A10 uses current signed distances
and an independent lower-arm phase control. Older observations below are retained
as historical evidence, not current acceptance.

Private continuation: guide35A4 received a qualified foundation-only artistic
pass on two wearers, but the filled native35A5 prototype failed the full-surface
review. Its per-section shape law broadcasts the side-boundary bow into a chest
shelf; the reused front meridian retains excessive lower fullness. Independent
checks confirm native rim/G1 interpolation, but also a two-row material station
mismatch and 8.8798mm actual-mesh body clearance against the 11.5mm requirement.
It is a diagnostic open mid-surface, not a solid or integrated generator output.

The next private35A6 experiment replaces fixed interior section/jet authority
with bounded whole-face fairing. It has one default-wearer trial, a 20mm
displacement trust region and eight contact rounds, without weight or preset
sweeps. The old rim is retained only to test surface compatibility, not treated
as permanently approved. Source remains frozen35T while these private native
prototypes are reviewed. Latest evidence and restart instructions are in the
local breastplate-review/RESUME.md; no all-morph or final artistic pass exists.

That35A6 experiment has now failed as well. Its finite fairing problem and
controlled contact continuation solve correctly, but the final full mesh still
has only8.2003mm clearance at an unsampled waist point and the broad reference
review rejects the whole shape. A conditional convex-front compatibility check
also contradicts exact interpolation of several retained rim points. All failed
native artifacts and review evidence remain.

Private35A7 reversed that authority: one elliptical style loft owns the face,
with neck/arm boundaries trimming its material domain. Its first solve failed.
Independent diagnosis found that a globally monotone angular arm trim imposed
an unintended depth-profile restriction and could not form the needed middle
scoop. The initial width measurement also included connected upper-arm material.

Private35A8 corrects the trim to two physical-C1 spans through a reference-guided
middle angle, and initializes width from the usable underarm reference. Its
first two subproblems converge, but the third stops without convergence; the
final actual mesh has ten triangles below the unchanged11.5mm clearance target.
Both the last converged and final outputs fail broad artistic review for upper
shelf/tubular or forward-standing form. No numerical, artistic, or family pass.
The next proposed diagnostic replaces historical local collision planes with
current signed body distances, after a stale-plane direction conflict was found.
Source remains35T; these private prototypes do not change the running generator.

Private iteration35T repairs the whole-front guide's transverse body-support
constraints using the retained radius and coronal center. The crown40 exports
for the three tested wearer/rigidity cases remain byte-identical to34C and
therefore retain its failed overall artistic status. A crown20 default-wearer
export now passes the previously failing support gate and produces a mesh;
the second wearer's crown20 export fails the earlier joint-profile solver,
before this guide is fitted. No solver tolerance was relaxed. This is a scoped
parameter-correctness improvement, not upper-boundary or all-morph acceptance.
The next upper-boundary experiment is guide-only: measure a torso-side opening
endpoint and author the complete armhole-to-waist course before integrating a
new surface. The prior restricted upper-arm guide failed artistic review.

The full-boundary35A2 witness also failed: a side-first tangent and a global
projection-curvature restriction forced the default arm curve through the body;
the other wearer exposed incompatible directional padding in the torso-arm gap.
Guide-only35A3 therefore tests a sparse body-derived interpolating network with
jointly owned tangents and measured lower-side/waist references. Its selected
points pass independent 11.5mm full-body distance checks (10mm requested gap plus
1.5mm half-wall), while their initial normal offsets retain a separate 5mm
authoring margin. Point tests do not certify connecting curves or the shell.
No new boundary has been integrated into the generator by these guide studies.

The whole-front trial adds `BREASTPLATE_DIAGNOSTIC_WHOLE_FRONT=1`
to the simple-rim/joint-profile configuration. It authors the front profile over
the occupied waist-to-neck height and connects it to the retained shoulder
region with an explicit curvature-continuous transition. Radius and coronal
center remain the controlled baseline. Iteration 34C adds the minimum
body-constrained transition correction that preserves both endpoint position,
slope and curvature. It uses the same flag and does not change the main guide
or the shoulder region above the transition. All three tested actual exports
pass scoped independent final-mesh checks. Fresh34C artistic review fails for
overall proportions and upper-boundary fidelity, most strongly the additional
wearer's scooped arm opening and raised shoulder approach. This remains a
diagnostic path, not an accepted ordinary generator or all-morph pass.

Historical comparison: actual33a passed the tested geometry checks but failed
broad reference review for its main volume and shoulder tips. The whole-height
front in 34B passed a fresh broad review on the default wearer at both tested
rigidity settings, with shoulder seating and proportional reservations. Its
second wearer failed an upper-transition body floor; 34C addresses that specific
failure without relaxing the floor. These partial results do not certify the
full supported body family or exported/runtime morph interpolation. The later
fresh34C critic disagrees with34B's default pass despite byte-identical geometry;
both verdicts are retained, and the overall result remains unaccepted.
Diagnostic33/33a tests `BREASTPLATE_DIAGNOSTIC_SIMPLE_RIM=1` with the existing seated
shoulder-band settings. It authors a straight shoulder guide and a separately
descending arm-opening curve with an intentional outer corner, instead of
inheriting the queried body's small rim undulations. The requested guide width
and seat are unchanged. This is an unaccepted comparison, not the ordinary
generator or a claim of all-morph validity. The fitted metal may still deviate
from its guide; actual body-included renders and independent audits are required.

The ongoing angular-loft breastplate remains gated by
`BREASTPLATE_DIAGNOSTIC_ANGULAR_LOFT=1`. The independently selectable
`BREASTPLATE_DIAGNOSTIC_SURFACE_PREVIEW=1` skips production mesh optimization;
leave it unset for a production-quality test. A preview is not a production
acceptance result.

The current upper-surface experiment additionally enables
`BREASTPLATE_DIAGNOSTIC_GLOBAL_UPPER=1`. Its openings trim a global surface
instead of defining a separate interpolation height for every meridian.
For controlled comparisons, `BREASTPLATE_DIAGNOSTIC_UPPER_RADIUS_MODE` accepts
`grounded` (default) or `minimum-bending`, and
`BREASTPLATE_DIAGNOSTIC_UPPER_GUIDE_MODE` accepts `landmarks` (default) or
`dense`. The radius modes change both fitting and conservative segment bounds;
they are policy alternatives, not merely two regularization strengths.
These diagnostic choices have not been promoted to the ordinary generator.

`BREASTPLATE_DIAGNOSTIC_UPPER_DEPTH_MODE=elliptical` selects an exact elliptical
cross-section alternative (`bernstein` remains the default). It constrains depth
radii to remain positive and the center meridian to avoid a bowl through the
neckline. Those limited continuous properties do not certify full surface
convexity or a valid triangulation; the surface and mesh are reviewed separately.

`BREASTPLATE_DIAGNOSTIC_METRIC_PANEL=1` selects the new surface-metric interior
layout (requires the angular surface). It retains the authored boundary and
uses derived inward sites with constrained triangulation instead of paired
boundary/rail strips. Its dump records vertex roles and site recipes. This is
still an experimental mesh path; leave preview unset when testing the unchanged
production quality gate.

`BREASTPLATE_DIAGNOSTIC_JOINT_PROFILES=1` adds the whole-torso profile experiment
to the angular/global/elliptical path. It fits lower and upper radius/depth
profiles together with curvature-continuous joins, then constructs the derived
arm-opening return directly in angular coordinates. The return covers both
armhole and adjacent side-edge stations above the fitted seam. The old fit is
computed once as a temporary A/B contact-error baseline; it is not the final
lower surface. Fresh body support is sampled after the boundary is rebuilt.
The authoritative dump is `joint_profiles`, marked
`kind: joint-torso-elliptical-v1`; legacy lower coefficients in that dump are
explicitly baseline data and must not be used to reconstruct the new field.
Pre-solve radius/depth constraints and physical sample provenance are saved
alongside it. This remains diagnostic: smooth analytic joins alone do not
certify the sampled boundary, triangulation, collision clearance, or appearance.

With joint profiles enabled,
`BREASTPLATE_DIAGNOSTIC_SHOULDER_BAND_FRACTION=0.30` selects the measured
shoulder-band experiment. The fraction controls a target arc length on a sparse,
smoothed, normal-padded clavicle-to-shoulder crest; `0.45` requests a wider band
through the same rule. The final fitted metal arc can differ from the guide
arc. The experiment preserves the neck endpoint and joins its curve tangentially
to the band, with fresh support fitting and detailed target-versus-actual
station dumps. Missing policy disables this experiment; invalid or unavailable
widths fail explicitly. This is not yet a public design-field/default change,
and successful exports do not establish reference fidelity or all-body support.

With the shoulder band enabled, `BREASTPLATE_DIAGNOSTIC_SUPERIOR_FIRST=1`
tests a straight-upward crest ray before the existing upward/anterior rays.
The same normal eligibility gates still apply. The dump records the query
policy, attempted hits and selected directions; this is a measurement-order
experiment, not permission to ignore body support or mesh quality. Unset the
flag for the archived anterior-first comparison. This improves measured guide
reach on the tested bodies but has not achieved artistic acceptance.

The next diagnostic adds `BREASTPLATE_DIAGNOSTIC_SHOULDER_SEAT=0.125`
(requires band and superior-first policies). It moves each interior query
origin toward the first posterior body wall before sampling upward. The
unseated anatomical arc remains the width scale; a sparse curve over the used
shoulder interval is solved to the requested width. Origin, query, terminal
solve and width-residual evidence are dumped separately. This target placement
has been reviewed for a fitting trial; the resulting plate is not yet accepted.

`BREASTPLATE_DIAGNOSTIC_ROUNDED_U_CORNER=1` independently selects a rounded-U
neckline with an intentional corner at the shared shoulder-rim endpoint. It
requires the band policy. A deliberate boundary corner is distinct from a
disconnected or accidentally stepped mesh edge; tangent continuity is not an
acceptance requirement at this designed corner.

Preserve all selected environment flags alongside the design and wearer recipe.
Set `BREASTPLATE_ANGULAR_DUMP` to a fresh absolute JSON path to capture the field,
topology and fit diagnostics. Evaluate a copied executable when rebuilding in
parallel, and retain its hash. Successful diagnostic export does not certify
triangle quality, smooth convex shape, shell orientation, or all-morph support.

Waist width now changes the fitted lateral ease, subject to a minimum support
reserve. Rigidity changes surface fairing without relaxing body clearance.
Both responses have been measured on one wearer; that is not validation across
all body types or parameter combinations. The combined angular/global/elliptical/
metric-panel baseline (iteration 28) passes independent main-mesh checks on the matched
wearer at default and minimum rigidity and on one additional body fit. Earlier
broad shape-family artistic passes were superseded by a clarified reference-
design review: all three iteration-28 cases failed design fidelity, chiefly at the
shoulder/arm-opening design and, on the additional body, upper-chest surface
flow. Different wearer proportions remain allowed. No all-body acceptance or
default promotion follows from these results. A directed-edge audit found and helped
repair inconsistent skirt rim winding: the three tested exports now have
opposing edge traversal throughout both closed components. The repair changes
only rim winding, not vertex data or wall diagonals. Undirected edge incidence
alone must not be treated as complete solid-shell acceptance. Localized upper-side
curvature remains a regression watch. Wrap response and full signed morph
export/runtime behavior remain unfinished. Parameter-file support makes these
tests reproducible; it does not itself establish geometry or runtime acceptance.
