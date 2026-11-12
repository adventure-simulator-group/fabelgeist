# Independent checkpoint 36 evidence review

`REPORT.json` records the three qualified review findings, historical reader
and revision distinctions, original versus reconstructed member hashes, and
limitations. `verification.json` captures the independently checked pre-change
stage identities. `observed-public/` preserves the inspected public index and
qualification records.

The separately retained observed snapshot contains exact private originals and
must remain local; `snapshot-manifest.json` binds its bytes without publishing
those originals. Its earlier-publication links and the user-supplied original
059 extraction are separate dependencies. The review is tied to the observed
index hash and is not an evaluation of any later correction.

`python3 verify-review.py` checks this review packet's own manifest. To
reproduce the wider read-only checks, run `python3 verify-observed.py
SNAPSHOT_DIR ORIGINAL_059_EXTRACTION_DIR PUBLICATION_DIR`. Both programs only
read evidence and print their results; neither invokes product code or another
packet's verifier. Hashes and receipts do not prove product behavior or actual
personal display.
