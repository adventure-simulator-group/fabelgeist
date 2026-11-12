# Independent checkpoint 35 packaging review

`REPORT.json` explains the findings and limits. `PROPOSAL.json` is an additive
correction proposal with exact member locations, source identities, safe prose
fragments, and stored versus uncompressed hashes. `EVIDENCE-IDENTITIES.json`
binds the reviewed stage and archive bytes.

`verify.py` is read-only. Set `CHECKPOINT35_STAGE` to the retained checkpoint
stage and `EXTRACTION_059` to the existing original extraction for the optional
independent comparison. The compact packet does not copy the archives or the
private originals. Those originals contain personal paths and must stay local.
`CHECKS-V3.json` is the last captured check; earlier checks are retained.

The proposal deliberately contains context-confirmed slash-delimited prose
that the old broad matcher would mistake for paths. Any publication check
should allow only the documented exact contexts and continue checking other
paths and credentials.
