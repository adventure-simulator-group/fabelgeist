# City-streaming registration privacy proposal

This is an additive, evidence-only proposal for two already-published
historical registrations. It contains redacted candidate bytes and the exact
historical assessment and verifier text, but **not** the original registration
gzip bytes. Those are kept separately under a permission-restricted private
sibling directory. The packet does not change publication or the user's
original extraction.

The original and reconstructed stored and decoded identities, the bounded
reference inventory, historic read attribution, integration plan, and explicit
limitations are in `REPORT.json` and `crosswalk.json`. An existing public cloud
representation differs from each original only at `/provenance/cache_path`;
this exact comparison was independently verified using the private originals.
The public-only verifier cannot itself prove the substitution.

`verify-city-crosswalk.py --publication PUBLICATION` checks the public
representations, historical transport manifest and assessment. Supplying
`--private-dir PRIVATE` additionally checks the original gzip and exact
substitution and reversal; optionally supply `--extraction EXTRACTED_059` for a
read-only comparison, and `--commit 56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e`
to check the individually recorded historical Git objects. No arguments cause
a write to any input. `verify-manifest.py` checks this packet's own files.

The corrected assessment and separately named full city evidence verifier are
under `candidate/`; their overlay was checked against the observed publication.
They are proposals, and the original assessment is retained as historic text
under `reference/`. A new evidence driver must exclude the old city check and
coordinate the separately prepared baseline-059 successor checks before any
publication change. Commit 56f already exposes the original gzip blobs: a later
commit cannot revoke historical objects or prior copies.

No audit lens was closed, no credit was transferred and no product was built,
tested or executed.
