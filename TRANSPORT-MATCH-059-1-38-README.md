# Additive 059 transport identity qualification

`TRANSPORT-MATCH-059-1-38.json` records stored-only and corrected union counts
for the captured delegated checkpoint indexes. The occurrence file identifies
all 1,213 newly exposed gzip-to-059 matches by public archive, nested member,
original and published hashes, and original transport records. An occurrence
is a location, not a historical read or a unique finding.

The legacy opaque-container file keeps checkpoint 5 and checkpoint 7 inner
matches outside the indexed totals. Checkpoint 7's captured public nested tar
exposes five entries before an invalid header; a separately retained private
original was reviewed as 186 entries. A later repair of that public container
must have its own identity. The public verifier checks the damaged public
bytes and recorded historical relationships; it cannot independently verify
private originals, custody, or the original 186-entry content.

Once these files and checkpoints 36–38 are overlaid into the publication root,
run the evidence-only `python3 verify_transport_match_059_1_38.py` there. The
optional `--root PATH` selects another public root; `--stage 36=PATH`,
`--stage 37=PATH` and `--stage 38=PATH` select separate staged publication roots
before overlay. The verifier reads public metadata and archives, writes nothing,
and checks no private paths. It executes no product code.

Original 059 Git blob IDs refer to uncompressed transport entry bytes. Stored
gzip hashes, public redacted variants and private original verifications are
separate facts. Exact private originals and historical authors remain in their
own retained evidence; nothing here grants a read, unique-finding or checkpoint
3821 credit.
