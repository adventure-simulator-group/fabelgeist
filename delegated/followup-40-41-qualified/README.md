# Follow-up evidence transport

`CATALOG.json` identifies eight separately attributed public archives. The
source packets cover historical roles 2629–2642; their findings, source reads,
reconstructed records, old and working baselines, and open questions retain
the qualifications inside each packet. Original checkpoint 3821 remains
unrecovered, and no lens is closed by this transport.

`central-13l` is the coordinator's bounded cloth review. `review-13l` retains
the earlier independently reviewed version and reviewer involvement; the
final wording correction is separate. The source crosswalk and public verifier
bind the coordinator's five source files to the cloth packet's Git proofs.

Run `python3 -B verify-archives.py` for read-only verification of the archive
and manifest bytes, source crosswalk and registered coordinator displays. The
embedded packet verifiers and same-machine receipts preserve their own deeper
semantic checks. Original private bytes are retained separately on one machine;
this public archive check cannot replay their redactions or establish privacy
across unscanned encodings or earlier Git history. No product behavior was run.

This separately named successor replaces only the GPU reflection archive, its
receipt and stage metadata. It withholds individual path-value and path-prefix
digests, while preserving whole-file identities and restricted exact originals.
`DIGEST-OMISSION-QUALIFICATION.json` records scope and count distinctions.

`python3 -B scan-private-digests.py TARGET --private-capture PRIVATE_DIRECTORY`
derives the restricted corpus at runtime and prints only counts and sanitized
locations. It is bounded and does not constitute complete privacy clearance.
Git administrative metadata and history are excluded; this scans only the
specified working tree, including untracked files.
