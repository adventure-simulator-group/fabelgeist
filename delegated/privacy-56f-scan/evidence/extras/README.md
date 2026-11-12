# Additional five-record reconstruction candidates

This is a local, reviewable proposal. It does not alter publication or the
historical records. `reconstruction-index.json` identifies the five originals,
changes, path-free replacements, both SHA-256 and Git blob identities, and
original custody relationships. The two gzip candidates already exist in the
committed cloud evidence; their only decoded JSON difference is the stated
cache-path value. The archive candidate is already present within a committed
reconstructed evidence bundle and must not be represented as the original
registry package or its checksum. The exact original archive remains separately
identified. Historic originals remain unchanged locally.

Run `python3 verify-candidates.py` for read-only candidate integrity and the
bounded affected-field path checks. In a repository containing the pinned
commit, add `--originals` to also verify original Git blobs, the byte-preserving
transformations, the historical JSON semantic comparison, and the archive
member and metadata comparison. `python3 verify-manifest.py` checks the packet.
These checks are scoped to the documented path types and records; they do not
prove general privacy or that all evidence consumers have been updated. No
product, package, fixture, shader or test is executed.
