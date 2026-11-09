# Registered source object recovery

The manifest lists 268 source identities transcribed from historical printed
registrations. Each Git revision, repository path and SHA-256 was checked
against local Git bytes; 264 hashes are distinct. A verified source blob does
not establish the completeness or validity of its registration, receipt,
read attribution, audit credit, or current revision.

Run `python restore_sources.py REPOSITORY DESTINATION` with a Git repository
that contains the recorded commits. The destination must not exist. The tool
creates content-addressed files in `objects/`, verifies each source hash, and
leaves `RECOVERY-INCOMPLETE` on failure. It does not alter the source repository.
A full clone or additional fetch may be needed to make historical commits
available. Review manifest provenance and the audit protocol before use.
