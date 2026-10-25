# Complete the #774 bespoke-type migration

Status: reconciliation and execution plan; source implementation has not begun
in this continuation. All final clearances remain outstanding.

## Verified starting point

The supplied handover is an incomplete audit transfer, not a completed
thirteen-lens audit. Its STATE.json is later than older completion summaries.
Our 124 findings and Adler’s 1,002 finding, support, alias and qualification
records are inputs, not additive counts of distinct defects.

- Our audit: merged #774 `13fc9dcc71ab879de364117e5462310673e43210`,
  same tree as final PR head `6223320d09f4691b6de54382d9709b233274d435`.
- Adler’s frozen PR: `6533e444fa37552927070cc8d8190907fd3b413e`;
  base `3a50b708381ad00c116445c2c999a49c3943088b`.
- Preserved P16: `039329518e264f19206cf5b0b98d177433f40c17`;
  P17: `90a422a453cd990ebc7f0ecf2ad5ed3e2d57d696`;
  rig quality: `cbbbca08c89a8768926799371f54cf6057312e5e`.
- GitHub main verified at `990430b77a9f63548ffcb642074185f61ddb3322`.
  It is the prospective implementation base, not an audited substitute for
  frozen variants. Recheck before creating an implementation checkout.
- Four `pr-working` overlays are recoverable from
  `36c97923da8ee84d0ab8994e7cfedf6a0cdea641`; exact byte fingerprints are in
  `assignments/qualifications/pr-working-overlay-records.json`.

See the reconciliation ledger and crosswalks for evidence and qualifications.
A source hash establishes provenance, not semantic acceptance.

## Execution gates

1. Finish source reconciliation and all initial lens inventories. Recover the
   exact source variants and overlays, compare their material contracts with
   merged/current main, and preserve every older pending trace. Retire a
   finding only with source evidence for its actual defect. Do not inherit a
   lens verdict from a different snapshot or from another lens.
2. Agree canonical contracts and one owner for each shared function. Update
   the exact claim records before editing. Reconcile overlapping remedies into
   one repair and credit all affected families without closing them by proxy.
   Direct human authorization now assigns the connected GPU claims to #780
   and #826; their exact split is pending. Unrelated families are not transferred.
3. Implement complete connected increments in dependency order. Recover P16
   and P17 semantically against current main; do not replay stale branches.
   Follow unchanged callers and receiving bodies, retained state, errors and
   publication paths. Reopen the family if a newly discovered gap appears.
4. After source completion, perform formatting and size/quality cleanup, then
   publish for hosted checks. Use existing tests only. No local project checks,
   new cases, assertions, helpers, doctests, weakened oracles or baseline hiding.
   Do not push while a known connected migration gap remains.
5. At each resulting revision, repeat full thirteen-lens waves until every
   lens is exhaustively covered and has no actionable unresolved finding.
   Targeted rechecks are useful evidence, not complete waves. Preserve source
   variants and qualified negative evidence when revalidating carried findings.
6. Run independent Code Review, Rule Review and Comment Review. Address their
   findings and repeat affected audits/reviews. Final acceptance requires all
   thirteen lens verdicts, all three reviews and required hosted checks on the
   same final pushed SHA. A later change invalidates affected earlier evidence.

No merge or completion of overall Goal #726 is part of this plan.

## Work preservation and boundaries

The original stash and four unrelated Cargo lint changes remain untouched.
Do not integrate Adler’s independent GPU work or other PRs implicitly. Use
canonical #779 launch/byte owners, #780 counts/layouts and #826 GPU receivers;
keep #895’s ZIP/NumPy owners coordinated dependencies rather than duplicating
those migrations. Keep tactical tick state in transient Bevy authority. Apply
the repository’s clean-schema policy, generated-client rules and documentation
boundaries. Owning error enums remain with their responsibilities; a shared
operation error must not become a universal file for unrelated leaf failures.

## Current status

Crosswalks and connected work batches are being reconciled by three read-only
investigators. Their final outputs will be linked here before delivery.
