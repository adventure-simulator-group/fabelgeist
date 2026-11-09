# Upstream source locators

Four reconstructed source-copy candidates match files at the pinned public
SpacetimeDB revision after appending one LF byte. The manifest records the
upstream paths, Git blob identities, source SHA-256 hashes, candidate hashes
and transcript ordinals. These are historical candidate bytes, not evidence
of current source coverage or audit credit.

Clone the upstream repository at the pinned revision, then run
`python restore_upstream.py REPOSITORY DESTINATION`. The destination must not
exist. The tool verifies the source bytes and transformed candidate hashes and
leaves an incomplete marker on failure. It does not execute the source or
modify the upstream repository. The public handoff stores locators rather
than copying the third-party source files.
