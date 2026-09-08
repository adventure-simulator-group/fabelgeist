# Character body equipment: modeling and review

These instructions govern body-equipment shape generation, fitting, and visual
iteration in this crate. They also serve as the shared workflow for equipment
work in the character creator. Documentation-only and unrelated maintenance
changes do not require a modeling or artistic-review cycle.

## Establish the target and scope

- Identify the user-approved reference images, intended geometric relationships,
  adjustable features, supported bodies, and acceptance milestone. Explicit user
  requirements take precedence over ambiguous details in an image.
- Judge the reference design adapted to the wearer, not exact pixels or identical
  body proportions. Merely producing a plausible member of the equipment category
  is insufficient. Do not excuse every design difference as necessary fitting.
- For shape-only work, exclude decoration, textures, colors, material matching,
  straps, and other details outside the request. Distinguish a deliberately sharp
  edge from unwanted faceting or a crease in an otherwise smooth surface.
- A simplified or generated reference needs user approval before replacing the
  target. Generated images can clarify intent, but are never evidence that the
  implemented mesh works.
- Distinguish finished equipment from surrounding placeholder clothing. Expected
  placeholder shape defects do not block acceptance of a separate finished item;
  still check data validity and interactions relevant to the requested item.

## Explore the representation before polishing it

Use one implementor for the ordinary reproduce/fix/render loop. The coordinating
agent may implement directly; independent reviewers do not edit geometry. Routine
reversible experiments need no additional approval gates beyond the user's scope.

1. State the geometric hypothesis and the visible mismatch it should address.
   Reassess whether the current representation has enough independent shape
   control, rather than assuming the next parameter tweak will solve the problem.
2. Build the smallest useful candidate and render the actual generated mesh on
   its body. Establish that a coarse representation can express the target before
   building an elaborate fitter, optimizer, or certification system around it.
3. Review meaningful candidates using the independent roles below. Internal
   debugging steps need not each trigger a full review panel.
4. Keep local adjustments while they produce substantial improvement. After two
   substantive attempts leave the same dominant defect, compare a materially
   different representation or control structure before more tuning. This is a
   prompt for judgment, not a mandatory restart counter; record the evidence for
   continuing or changing direction.

Separate anatomical placement anchors, artistic shape controls, and clearance
constraints. Canonical UV correspondence identifies body regions; a sampled point,
ray hit, or skin-weight transition is not automatically a semantic landmark.
Confirm its anatomical meaning in body-visible views. Record query origins and
directions when diagnosing projection or fitting behavior.

Treat topology as a design choice. Give openings, corners, seams, and major shape
regions appropriate sampling and independent control; do not assume cloned body
triangles or a highly warped rectangular grid will express every boundary well.
Test whether moving a boundary unintentionally changes the bulk surface. Compare
options such as organized cages, patch networks, or smooth carrier surfaces with
separate trim domains when controls remain coupled. No particular surface basis
is required, and a new architecture is still a hypothesis, not a commitment.

## Evidence package

- Supply the approved reference alongside current body-included renders to each
  visual reviewer, preferably in a clearly labeled comparison board. Reviewers
  must actually open the images, not rely on filenames or implementor summaries.
- Include front, side, and three-quarter views, plus rear or local views when
  needed to judge coverage, seating, or interactions. Confirm the camera actually
  changed and the equipment is large enough to inspect. Isolated mesh views and
  overlays supplement, rather than replace, body-visible views.
- Preserve a reproducible candidate: source revision or builder snapshot, body
  recipe and morph weights, design parameters, relevant flags, exported mesh,
  renderer/camera settings, and artifact paths or hashes. Identify whether the
  evidence is a prototype, ordinary generator output, or installed runtime asset.
- Cross-check ambiguous highlights using another view or diagnostic shading.
  A highlight alone does not establish a geometric dent, fold, or intersection.

## Independent review roles

For meaningful shape iterations, use a broad reference critic and a separate
known-failure regression critic, objective checks, and a coordinating arbiter.
Reviewers should challenge the result, not defend the implementor's effort or
invent issues to sound adversarial. Do not pass along another reviewer's verdict
before an independent assessment is recorded.

### Broad reference critic prompt

Give this prompt with the approved reference, current renders, and a short scope
statement containing user requirements and exclusions, not known-defect hints:

> Actually view the approved reference alongside every supplied current
> body-included render before reading code, measurements, implementation notes,
> previous candidates, or other reviewers' conclusions. Identify what you viewed.
> Does the modeled equipment accurately represent the reference design's shape,
> adapted to this wearer? Describe the major volumes, surface flow, boundaries,
> and relationship to the wearer as a coherent whole, then compare them with the
> reference. Category membership, technical validity, or improvement is not enough.
>
> Allow different wearer proportions and reasonable fit adaptation. Observe the
> supplied scope exclusions and explicit geometric intent. Do not infer defects
> from a metallic highlight alone or invent problems. Separate visible observations
> from hypotheses about their cause and from uncertainty about fitting.
>
> Return PASS, FAIL, or UNCERTAIN with confidence, at most three consequential
> mismatches ranked by visual impact, evidence tied to named views and regions,
> and meaningful reservations. If evidence is inadequate, name the additional
> view needed. Promising exploration can still FAIL acceptance. Do not claim mesh
> integrity, numerical clearance, runtime behavior, or all-body support from images.
> Do not demand a particular implementation or prescribe only small tweaks.

Keep this prompt broad; do not accumulate a checklist of the current model's
failures in it. After the initial assessment, the critic may compare an earlier
candidate for progress. At acceptance milestones, use a fresh reviewer without
prior implementation/review history to check for anchoring or checklist overfitting.
Repeated review by the same agent is not a fresh unprimed assessment.

### Known-failure regression critic

Provide the same images plus the item's observed failure history and user
feedback. Ask for evidence that each relevant regression is present, absent, or
unresolved. Keep item-specific examples here, not in the broad critic prompt.
An absence of known defects is not proof of reference fidelity.

### Objective checks and arbiter

Run objective checks independently of artistic verdicts, with explicit units,
thresholds, tested configurations, and limitations. Distinguish sampled evidence
from continuous guarantees, and a conservative warning predicate from proof of a
specific defect. Evaluate the actual generated mesh, not just its control surface.

The coordinating agent inspects the images and arbitrates all evidence. Do not
average away a consequential failure or treat a reviewer PASS as automatic
acceptance. Resolve conflicts using targeted views or measurements; unresolved
evidence stays unresolved. Technical validity is not visual acceptance, and visual
acceptance is not production readiness. Retain exact reviewer outputs and the
reason for the final decision, including changed thresholds or superseded verdicts.

## Proportionate validation and durable state

- During exploration, check finite/index-valid geometry, gross intersections,
  rendering errors, and triangle quality where it affects shape. Add representative
  extreme bodies once the shape direction is credible; avoid optimizing one wearer
  or one camera while claiming generality.
- Before production promotion, validate the agreed supported body and parameter
  scope, thickness, clearance, winding/topology, component interactions, and relevant
  tests. Where morph support is part of that scope, cover the supported catalog,
  fixed connectivity/correspondence, and representative blends and extremes. State
  baseline/zero-weight semantics explicitly. Deferred morph work is not a completed
  capability, but is not a new gate for an explicitly base-only milestone.
- Validate runtime integration on the installed artifact and relevant poses or
  animations. A static external render or successful process exit is not runtime
  visual acceptance. Report precisely what was exercised and what remains untested.
- Keep a concise current checkpoint with candidate/render paths, present defects,
  current hypothesis, next action, and acceptance scope. Save meaningful candidates
  and why each was kept or rejected; do not turn every tool correction into a new
  contract or append an ever-growing experiment history to this guide.
- Historical reports remain evidence, not mandatory gate queues. Reuse useful
  validation tools, but update parameterization-dependent measurements when the
  representation changes instead of preserving obsolete coordinates as requirements.
