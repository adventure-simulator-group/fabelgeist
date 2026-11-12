# Privacy review evidence transport

`REPORT.json` states the committed-tree inventory, reconstruction decisions,
recorded corrections, registry identity and limits. `transport-map.json` maps
each frozen source file to its public representation with both byte counts and
SHA-256 hashes. Captured Python files are source text with a `.py.txt` suffix;
these historical scripts are not the verifier for this transport.

Five already reconstructed JSON candidates are individually gzip-compressed.
Their decoded identities remain distinct from both their transport bytes and
the private committed originals. Two redacted registration gzip files and the
reconstructed package archive are copied without modifying their nested
contents. The package reconstruction does not have the registry checksum.

The exact private source objects and original frozen scripts are held in a
separate local-only capture. Historical manifests in `evidence/` bind their
original packet bytes and may refer to original paths or original script
identities; they are evidence, not validators for the reconstructed transport.
This transport includes a new verifier and root manifest for its public bytes.

Run `python3 verify-public-transport.py` to check the mapping, all transmitted
file identities, the five JSON candidate payloads and markers, the additional
candidate identities and selected archive member, the registry metadata, and
the manifest. Verification is read-only and requires neither private paths nor
product execution. It is not a full replay of the prior scan, a general privacy
or secret clearance, or a live registry download.
