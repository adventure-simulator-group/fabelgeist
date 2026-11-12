# q2945 bounded Lens 2 evidence

This reconstructed public successor retains exact selected source blobs, raw
Git tree objects, independent display transcripts, failed Git diagnostic
stderr, two already published ledger rows and a privacy-reviewed assessment.
The report and historical summary are projections; original historical queue
entries and raw commit metadata are separately held and not included.

`python3 verify.py` checks this manifest, source blobs, raw tree path chains
and all registered displays. It cannot bind the tree roots to commits or
verify original queue membership, original receipts, the full ledger or the
historic lens-mapping source without separately held restricted evidence.
Supply `--private DIR` to verify the sealed restricted packet and its complete
source/receipt proof; add `--anchors DIR --ledger FILE --historic-values FILE`
for full original-queue, immutable-ledger and historical mapping replay.

Only per-file identity is established across 6533, e275 and 95e; the old
checkpoint 3821 and outstanding 73 nontext/no-positive-EOF obligations are
not reconstructed or closed. This bounded scan and evidence replay do not
execute product code or establish runtime behavior.
