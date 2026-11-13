# Bounded recovery-object diagnosis

Reviewer attribution: wrapped_scope. This is a source-independent analysis of
Git objects and evidence-process records; it conveys no new product-source or
lens read credit.

The first successor recovery matched 263 captured files, then its first
successor guard failed while resolving original historical checkpoint 059.
Its two later successor checks did not run. In the recovery checkout, commit
`eb407672813424d64ec31f208193969f5a62620c` and its four path trees were
missing, although the historical queue blob was already present. The pinned
source commit `95e63628f9d70b8dd6e6fede64334696c6995009` and its tree were
also missing. The b111 and a422 objects and the selected report, pending and
assessment paths checked by this reviewer were present; this was not a full
reachable-object audit.

A process-local publication object-store alternate resolved both missing
commits. The selected historical queue then matched its registered SHA-256
`2e1f4430edb375adff24b6b46d434ac58bf5fd18c0d9dda4cf92fe215e5cfe08`.
The coordinator separately reported that a later alternate-dependent replay
passed all three successor checks; this reviewer did not independently rerun
that replay. Its same-machine object-store dependency is part of the result,
not independent or standalone custody. The alternate does not persist merely
because the environment was used for one process.

The recovery clone also held the historical commit under a direct ref, and
its selected queue independently matched the same hash. The publication clone
held the pinned source commit under a direct ref. These stores and their refs,
or separately verified equivalents, are needed for later replay. A lookup of
the old handoff-bundle path failed; the evidence-only payload Git store did
not contain the historical commit. No deletion or explanation is inferred.
The first failure, later success, original 059 source, reconstruction, older
summaries and missing original checkpoint 3821 remain distinct.
