# q2317: cooking preview and assembly, bounded source reconciliation

## Disposition and evidence class

This is a newly assessed, source-only follow-up to the recovered 059 queue
entry 2317, `PL13-247-COOKING-PREVIEW-TARGET-CONTRACT-PENDING`. It is not a
closure of that question, a complete lens, or a count of defects. The selected
original and path-redacted reconstruction are byte-identical; their parent
queue files are distinct. The original 059 entry and exact historical
registration, read, verification, root adoption, proposal and source object
are retained under `history/` with hash receipts. The prior reviewer attributed
a personal full-body read of `food/cooking.rs:1-868` to frozen revision
`6533e444fa37552927070cc8d8190907fd3b413e`. That attribution is preserved;
it is not fresh 95e body credit or a recovered checkpoint 3821 final record.
The original proposal marked current reconciliation, global clearance, and
product execution false, and its queue item remains open.

The working baseline is the centrally pinned revision
`95e63628f9d70b8dd6e6fede64334696c6995009`. Its and the frozen revision's
captured `food/cooking.rs` have the same blob and SHA-256, and both commits
point to the same `food` subtree; their root trees differ. The captured core
`food.rs` bytes also match across revisions. Sixteen exact source paths are
bound by commit-to-tree-to-blob proofs; these facts do not establish equivalence
of the commits, crate trees, historical reviewer attribution, or all files.
Current fresh display ranges are only those in `READ-LIMITS.json`.

## Current trace

* `crates/adventuresim-stdb-module/src/food/cooking.rs:819-868` defines a
  plain Rust `preview_cooking` function for selected, personal inventory
  lots. It passes raw catalog cooking safety and selected ingredient mass to
  the core checked-duration function. Its actual external or production caller
  remains unproved. A physical-worktree search is unpinned navigation only,
  and cannot establish global caller absence.
* The independently implemented web spit-roast draft display is a separate
  producer/consumer path. The selection template at
  `crates/strategic-web/src/templates/settlement/cooking/selection.rs:3-16`
  provides the displayed draft control; the surrounding fireplace layout and
  branch are in `templates/settlement/cooking.rs:93-115,181-185`. The
  strategic shell includes `cooking.js` at `templates/layout/shell.rs:103-115`;
  that script mounts on page and live-region events at `static/cooking.js:
  251-258`. This is static source evidence, not a runtime observation.
* The selected row writes unadjusted catalog `cooking_minutes` and mass
  formatted to four decimal places to attributes at
  `templates/settlement/cooking.rs:208-249`. `static/cooking.js:45-75,
  109-122,204-216` reads those attributes, computes setup plus raw maximum
  safety plus the batch term, and labels the result "min before chef-speed
  adjustment". No Rust `preview_cooking` call occurs in this captured JS file;
  that statement is not a repository-wide caller conclusion.
* The web ingredient route calls the loose-ingredient reducer
  (`routes/settlements/party/cooking.rs:430-483`); the reducer supplies no
  vessel and assembly selects Roast (`food/cooking.rs:85-102,281-291`).
  Assembly validates inventory and amounts and derives mass from the lot and
  selected fraction (`:313-390,420-421`). It applies a 0.75 Cut or 0.50
  Ground factor to raw safety with ceiling and a minimum of one
  (`food/cooking.rs:382-390`, `adventuresim-core/src/food.rs:11-13,84-88`).
  A preparation reducer and authority code can select a carried source lot,
  validate the transition and update its preparation (`food/preparation_authority.rs:350-403`,
  `food/ingredient_preparation.rs:285-303,535-587`).
  This establishes a source path for a producer subject to its guards, not a
  demonstrated reachable user state.
* Core's pre-chef baseline takes the maximum supplied safety and a batch term
  based on total mass, and rejects empty safety and invalid or over-25 kg mass
  (`adventuresim-core/src/food.rs:371-392`). Chef adjustment reduces only
  overhead, not safety (`:394-410`). Thus, if an admissible staged prepared
  lot changes the maximum safety, the web display's raw-safety pre-chef
  calculation can differ from the assembly's pre-chef baseline; independent
  mass rounding and batch thresholds may affect the comparison. The label
  discloses the chef-speed adjustment but does not name preparation. Exact
  preview policy, admissible catalog/lot combinations, and any observed
  discrepancy remain open; no particular numeric outcome is asserted.
* Container cooking is a separate route and assembly path. It chooses method
  from the instrument and adds actual contained water before calculating the
  target (`food/cooking.rs:130-208,287-290,449-490`, route `:599-628`). The
  Rust preview has no corresponding vessel-water parameter, but the inspected
  web selection explicitly describes spit roasting and a separate vessel path;
  this evidence does not establish a displayed vessel preview or its mismatch.
* Assembly stores the resulting target (`food/cooking.rs:550-580`); projection
  and the fireplace page then present the stored dish target
  (`food/projections.rs:506-535`, route `:248-278,326-352`, template
  `:43-80,100-110`). That persisted-state display is distinct from the draft
  JavaScript preview and may reflect chef adjustment.

## Separate comment discrepancy

`food/cooking.rs:105-108` says a placed vessel consumes cookable lots at any
nesting depth. In the same function, `:150-156` says it intentionally sees
only direct contents and iterates direct `parent_object_id` edges. This is a
bounded source comment/implementation discrepancy; the intended contract and
any other traversal remain open. It is recorded separately from the preview
question, without an alias or new defect count.

## Relationships and limits

The 2026-11-10 95e ledger snapshot search is time-bound and literal. An initial
broader search incorrectly iterated the dictionary's keys; the retained
`LEDGER-RELATED-DIAGNOSIS.json` invalidates its zero results. The corrected
search uses all 1,034 `/newly_assessed_ledger/entries`; see registrations,
views, and candidate displays. `REVIEW-COOKING-WATER-EXACT-VOLUME-001`
(entry 82) concerns exact volume and floating-point conversion, and
`REVIEW-SIM-PROVISION-FIREPLACE-FOOD-001` (entry 185) concerns provisioning
custody and a separate fireplace lifecycle. They supply related context, not
proven aliases. Literal absence of the q ID or symbols does not prove semantic
absence. Do not assign counts or transfer earlier records into checkpoint 3821:
the final 3821 ledger and queue remain unrecovered.

Open work includes explicit preview accuracy policy, pinned global caller and
alias searches, actual admitted catalog and preparation states, exact
personal/party amount and custody behavior, complete native and vessel
traversal, and current ledger reconciliation. No build, test, product code,
fixture, product parser, or runtime probe was executed. Source test lines in the
captured files were inspected as text only. `READ-QUALIFICATIONS.json` retains
unregistered scratch metadata views and the defective search; they confer no
fresh source credit. Search results with exit status 1 indicate no literal
match within their registered physical-worktree scope, not exhaustive absence.
