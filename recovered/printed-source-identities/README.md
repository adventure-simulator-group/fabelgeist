# Additional printed source identities

These 118 exact revision/path/SHA-256 identities were extracted from recovered
printed audit values and independently checked against Git. They add 91 source
blobs beyond the separately restored registered-source set. Their source value
collection and normalized value hash are retained in the manifest. Verified
source bytes do not establish valid read attribution, audit credit, current
source coverage or final checkpoint state.

Run `python restore_sources.py REPOSITORY DESTINATION` against a repository
containing the recorded commits. The destination must not exist. The tool
checks every SHA-256 and leaves `RECOVERY-INCOMPLETE` on failure. It does not
modify the repository or execute source code.
