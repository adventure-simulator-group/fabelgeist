# XPBD particle evidence transport

This local stage reconstructs the frozen source reconciliation for original
roles 2648–2649 and its separate prior-reader attribution addendum. It preserves
the 774 rig records, the 059 queue snapshots, prior reviewer scopes, and the
corrected distinction between working and locally committed baseline captures.
See `PROVENANCE.json` and the two reports under `bundle/` for the evidence and
remaining questions. No historical work is promoted to checkpoint 3821.

`EVIDENCE-IDENTITIES.json` binds original and reconstructed whole-file hashes,
redaction offsets and source-manifest identities. It records no digest of an
individual redacted path. Original manifests continue to identify restricted
original bytes, not reconstructed public members. The exact originals are
stored separately in an owner-restricted same-machine capture.

All six Git commit objects have author and committer headers redacted. Their
original Git IDs therefore cannot be cryptographically revalidated from these
public bytes. The preserved tree structure, exact leaves and original/public
crosswalk are checked separately; one older 059 queue leaf is also redacted.
The private verifier can check every exact substitution while the restricted
originals remain available. No prior publication or remote status is inferred
for the locally captured audit commits.

Run `python3 verify-public-evidence.py` for the read-only public evidence check.
Original packet scripts are retained as evidence text and are not this verifier.
Neither verifier executes product code or establishes source conclusions,
caller reachability, privacy completeness, runtime behavior or a finding count.
