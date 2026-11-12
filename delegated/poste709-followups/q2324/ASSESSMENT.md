# Bounded q2324 hypothetical-probe orientation review

Independent reader: `/root/mhr_character`. Current product source is pinned to
`95e63628f9d70b8dd6e6fede64334696c6995009`. This source-only review concerns
original 059 queue entry 2324,
`REVIEW-L11-CREATOR-HYPOTHETICAL-PROBE-ORIENTATION-POLICY`. It does not close
that queue entry at 95e or a lens, implement a repair, render, execute product,
tests or fixtures, or reconstruct missing checkpoint 3821.

## Historical identity and attribution

The original 059 queue is the version at publication commit
`56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e`, whole-object SHA-256
`2e1f4430edb375adff24b6b46d434ac58bf5fd18c0d9dda4cf92fe215e5cfe08`.
`QUEUE-2324-RAW.json` is its exact half-open byte slice `[10557431,10565525)`.
Its parsed content equals the displayed reconstructed projection and the
matching selected object in the later privacy-reconstructed queue at
publication `a9d979e4f4fd84b797da89a33263d78604fe9224`. The complete queues
are not byte-identical. The original queue is not embedded; independent
verification of its slice requires separately available exact source bytes.

The historical `parallel lens11_reconcile` reader recorded independent full
personal reads at **e275**, not 6533: underlayer proportions 1–134 (PL11-1122),
probe bounds 1–125 (PL11-1129), and creator proportions 1–79 (PL11-0239).
The original plans, body receipts, root adoptions and verifications are
hash-matched and retained under `history/`, together with the relevant bundle
proposal. The `segovb` root adoptions retain delegated attribution, explicitly
claim no root reread and record zero accepted unique-count changes. For
PL11-0239, plan `/rows/10` and body receipt `/rows/11` are different indices
in different lists; both identify the same source, hash and registered range.
Do not replace the historical reader with a newly inferred individual.

The original named-scope disposition describes quarter-oriented-area reserve
and hypothetical sampling as explicit policy, with no promise of nonadjacent
collision detection or full editor-range safety. It marked *that historical
named scope* closed while retaining dependencies and prohibiting any whole-tree,
all-lens, current-source, repair or runtime conclusion. Preserve that
historical status and its revision even while considering the separately
identified current conditional below.

All three named source objects have the same Git blob and SHA-256 at historical
`e2755a732c84e67296b5e3441a4259a84f461b2f` and pinned 95e. The recovered
probe-bounds object itself also matches the supplied 059 source-object bytes.
A filename-only search did not find SHA-named copies of the other two in that
extraction; this is not a claim of global absence. Their e275 Git bytes and
hash-matched historical receipts are preserved here. I independently registered
and displayed the full 95e bodies. Same-file identity is not whole-tree
identity, and my reads do not transfer the older reader's credit.

## Conditional nonfinite probe path

If a generated body's selected face is finite and nondegenerate, but a sampled
vertex of that face becomes NaN, the invalid-path safeguard does not actually
restore that vertex to its finite original. In
`underlayer_equipment/proportions/probe_bounds.rs:16–31,50–62`, nonfinite
sample coefficients make `AreaPath::new` fail and select fraction zero. In
`underlayer_equipment/proportions.rs:64–79`, that fraction is then applied as
`original + (sample - original) * fraction`; NaN multiplied by zero remains
NaN. A nonfinite difference caused by overflow is another unproved numeric
boundary. This is a symbolic source consequence under the stated inputs, not
an executed fixture, runtime failure or observed user impact.

One relevant input route is source-visible, but its supported end-to-end
reachability is not established. `fabelgeist-fbx/src/lib.rs:134–142,169–171,
430–483` decodes binary floating arrays without a finite-value check in those
windows. In `fabelgeist-mhr/src/character/skin.rs:97–149`, a NaN cluster
weight is not removed by the `<= 0` guard, can make the normalization sum NaN,
and is not rejected by the `total <= 0` comparison. The packed-skin constructor
checks joint bounds (`fabelgeist-rig/src/skin.rs:112–123`); it does not check
weights. The native upload keeps only `weight > 1e-5`
(`fabelgeist-mhr/src/model/upload.rs:126–146`), while the CPU probe's
`strongest_four` reduction can propagate NaN through its sum/division
(`fabelgeist-rig/src/skin.rs:79–99`). Thus these consumers can treat a nonfinite
weight differently. The precise condition that the earlier generated selected
face is finite, and all intervening asset, GPU, body and fitting admissions
succeed, remains unproved.

The current caller passes the resulting hypothetical shapes to underlayer
fitting (`underlayer_equipment.rs:58–90`; `underlayer_armor.rs:38–55`). In the
inspected fitting window, `device_underlayer.rs:157–190` checks matching
position/normal lengths and then uploads the sample positions. Earlier
`body.upload`, frame construction or `CutPlan` may reject first; their complete
behavior, backend handling and resulting garment/output are outside this
source-only review. A future repair direction is to reconcile finite-value
admission and invalid-probe fallback at the owning boundary after the audit and
separate authorization. No duplicate or unique defect count is assigned.

The existing 059 q167 and current ledger row
`ADLER-L11-MHR-SKIN-WEIGHT-ADMISSION` already concern nonfinite skin-weight
admission. Their original pr-working source hashes and attribution remain
separate; the 059 record leaves its revision field null. This review adds a
conditional probe-consumer relationship, not an automatic alias or an
independent historical finding. `/root/verifier_scope` independently observed
related finite-admission gaps in q2323; its separately sealed packet identity is
in `PRIOR-Q2323-REGISTRATION.json` and its read credit is not transferred.

## Face bounds and orientation policy

`fabelgeist-mhr/src/character.rs:118–147,175–205` checks vertex triples and
polygon structure but the displayed triangulation does not compare each face
index with the vertex count. `probe_bounds.rs:50–56` indexes both point arrays
directly by each face. This is conditional on such a mesh reaching the helper;
it is not proof of a supported asset or a demonstrated probe panic. The native
upload checks conversion to `i32`, not mesh membership in the displayed window
(`model/upload.rs:75–81`), and ordinary generation invokes MHR forward first
(`generation.rs:21–32`). The earlier `vertex_normals` path uses those face
indices in native tensor select/scatter (`model.rs:364–369,402–442`), which may
fail or behave differently before the later CPU probe. No backend or product
behavior was tested. The first-failure and producer questions remain open.
This is related to 059 q165 and `ADLER-L11-MHR-TOPOLOGY-ADMISSION`; its old
pr-working object is not asserted equal to the 95e body or counted again.

For finite, valid inputs, `AreaPath` computes an f64 quadratic of projected
face orientation, checks its stationary minimum across the path, and uses a
24-step search plus downward rounding of the selected fraction. The local
reserve and sagittal-gap clamp are policy on hypothetical samples, not proof
of whole-body self-collision freedom, every editor endpoint, or any runtime
product behavior. The f64 calculation and later f32 coordinate interpolation
also require an explicit numeric error/domain argument before claiming an
unqualified reserve for arbitrary inputs. No separate finite arithmetic
counterexample or intended stronger policy was established here.

## Scope and recovery limits

`READ-LIMITS.json` lists exact personally displayed windows and unread
complements. Three historical source bodies were captured and Git-hashed but
not personally redisplayed; historical reviewers keep their original body
credit. Test-source lines were inspected as text only, never run. Initial
output of `crates/AGENTS.md` was truncated, then its two halves were reread;
an initially oversized support range was corrected to verified EOF before any
display. The first q2323 report locator assumed `REPORT.json` and failed; the
metadata-only retry found `REPORT.md`. Those events are retained in the
registrations and `RULES-READ.json`.

The packet includes source Git-tree chains, but named commit-to-tree binding
requires independently available original Git objects; raw commit author
metadata is omitted. The full 059 queue, supported asset ingress, all
intervening validation, earlier native behavior, broader aliases, other lenses,
privacy beyond the bounded scan and independent-machine custody remain open.
