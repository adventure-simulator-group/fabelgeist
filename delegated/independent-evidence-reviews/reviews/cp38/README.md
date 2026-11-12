# Checkpoint 38 independent packaging review

`report.json` records the reviewer, conclusions, original and current role
identities, named source reviewers, qualifications and observed false-positive
redactions. `review-results.json` records the evidence-only checks and the
input hashes. The exact staged index, qualifications, progress snapshots, prior
manifest and committed/observed baseline copies are under `reference/`.

The three staged evidence archives and 22 exact changed top-level originals are
hash-linked but not duplicated here. Preserve the stage's private-originals
collection and archives separately; this compact review packet alone cannot
reconstruct all of their bytes. Public copies contain path redactions and six
prose false positives; exact originals have distinct identities. Inspect copied
metadata and any originals for private content before external publication.

`python3 verify-manifest.py` checks the frozen packet offline. It does not
verify live staging. `python3 verify-readonly.py` reads the stage, live
publication and original local packets without mutating them and prints fresh
results; that check depends on those inputs still existing and may observe a
later state. It does not invoke packet-local verifiers or product code. The
saved `review-results.json` represents the specified historical review time.
The capture/write scripts construct this packet and must not be rerun after it
is frozen. No product, macro, shader, fixture, build, test, probe or repair was
performed.
