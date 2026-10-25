# Frozen frame b874 Comment Review

Reviewed immutable HEAD `b874e611826c27085da897903cacfc31998d0210`, tree
`271b944ed1843f9e858c65da4947e854743e98ed`, against parent
`6b22364249ae1503fe070dd7b5ce64b8e6e0e25d`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`. All three changed files,
including the full current renderer, and their entire exact delta were read.
JSON records source hashes and exact unchanged affected-contract coverage.

No new actionable comment mismatch was identified in the frame/publication
repair. 6B-C1 is corrected: constructor docs distinguish returned host atlas
admission/allocation errors from device-handled native failures.

FrameProgress holds the candidate view, count, history and movement state.
Preparation resets only that candidate when the view changes or required
history is absent. Jitter, mean weight and destination history use the candidate.
The complete-view fast return preserves committed renderer state. Publication
updates last/count/history and clears movement only after queue.submit returns.

RecordedDeformations similarly records at most the three main/shadow/blend level
obligations, deduplicates levels and leaves unused levels pending. Its recording
path borrows the renderer immutably; acknowledgements become Current only after
normal submission return. An earlier Result exit or unwind leaves those
acknowledgements and temporal candidates unpublished.

This is source/API ordering, not a native acceptance or completion verdict.
Native failures still follow configured device handling; a returning handler
need not prove actual GPU acceptance. The new comments explicitly distinguish
submitted commands from completed GPU work. They promise neither a wait nor
rollback of preceding queue writes, resource creation or shadow-map updates.
Those distinctions match the actual boundary.

Finite frame admission, policy identity, history validity, device/source
provenance and atlas/readback contracts retain their exact earlier source where
accounted. Their previous complete or bounded coverage is carried with its
limits; this is not a fresh whole-project or whole-SDK review. Source ordering
keeps the new responsibilities and their documentation together, with existing
fixtures last.

The renderer fixture retains exact SHA256
`44c66d149be12de0828289a8461e243c235facb1032230263538ebc471cfda5e`:
the same two tests, validate helper and five assertions. All three shader blobs
are byte-identical. No tests/cases/helpers/assertions/doctests were introduced.

Required native strand/card/simulation-source/count/layout, producer, creator,
caller/cache and lock integrations remain open. Retained upload-prefix fit prose
still requires native-family revalidation, and atlas producers must consume the
new canonical API. None is waived by this frame repair.

No source edits, project execution/checks/builds/tests/scanners/generators,
runtime, push, hosted CI or external posts were performed. No compiled,
browser/hardware/artistic, performance, global-health, rollback or universal
allocation verdict is supplied. Earlier withdrawn assurances remain withdrawn.
This bounded checkpoint gives no full-PR or all-thirteen clearance; final
independent reviews and hosted gates must pass on the same final pushed revision.
