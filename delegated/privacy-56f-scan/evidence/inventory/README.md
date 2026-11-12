# Published evidence privacy review: commit 56f24f6

This local, attributed evidence report inventories the committed tree and
recursive gzip/ustar members under the recorded limits. It identifies 64
concrete path occurrences in ten top-level files, alongside 62 classified
examples or regex literals. `report.json` summarizes custody, historical
attribution, affected byte-verifier links, source identity, proposed
reconstructed variants, and limitations. `occurrences-classified.json` records
all 126 occurrence chains and hashes without copying the matched values.

The frozen initial pass is included unchanged in `firstpass/`; its earlier
nonrecursive scope is superseded only for the separately recorded follow-up.
Prefix, tail and revisit data and scanner sources are in `scan/`. The recorded
38 original budget exclusions were separately revisited. The extra suffix scan
overlaps the final tail and is never counted twice. `native/` preserves the
public registry index row, local-index row, lock identities and exact package
checksum relationship. The external web tool returned the public index through
its cache, but did not provide a successful static crate download.

The two reconstruction indices bind separate local frozen candidate packets by
manifest hash; this inventory does not copy those candidates. The exact
originals are committed Git objects, not restored 3821 evidence. The exact
native original also matched a local package cache at review time. A redacted
archive is a different representation and cannot claim the original registry
checksum. A later commit does not remove an earlier published commit.

Read-only evidence verification, run in a repository containing the identified
commit and project source revisions:

- `python3 verify-manifest.py` verifies all packet bytes, including the nested
  first-pass manifest.
- `python3 verify-evidence.py` verifies the tracked-tree count, each recorded
  occurrence chain and matched-prefix hash, classification totals, recorded
  revisit status and candidate identity links. It does not replay the full
  scan or establish that all possible path encodings were found.
- `python3 native/verify-registry.py` verifies the selected registry rows,
  committed original archive and 6533/95e lock identities. Add `--cache` on
  the review host to verify its recorded local cache state.
- `python3 verify-private-tokens.py` checks that selected discovered account
  prefixes and the two historical temp tokens do not occur in this packet;
  it requires the original Git objects and emits no private values.

Some retained scanner source names local temporary evidence paths. This is a
local report, not a general privacy-safe public release; a public copy requires
its own bounded review and reconstruction. No product code, test, build,
fixture, runtime probe, historical verifier, or cleanup was run for this
review.
