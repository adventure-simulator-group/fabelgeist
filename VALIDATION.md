# Recovery validation

- The original package was extracted from commit
  `eb407672813424d64ec31f208193969f5a62620c` without modifying its files.
  Its unmodified transport verifier checked all 36,487 logical files. On
  Windows a small adapter normalizes Git's root path spelling only after
  comparing the resolved paths; it does not bypass the original checks.
- The original restorer fully extracted and verified all 36,487 logical
  entries in a fresh, isolated, case-sensitive directory. The restored
  selector, ledger and closure queue were independently checked against the
  original manifest after writing. Historical helper paths remain unchanged.
- An independent fresh-clone `--verify-only` run of this package checked
  7,196 public progress files and all 36,487 original transport entries. It
  recorded that the later authoritative continuation was not recovered.
- A second fresh-clone run exercised `restore.py` end to end in an isolated
  case-sensitive directory. It verified the progress snapshot, restored all
  36,487 original entries, and completed without either incomplete marker.
  The restored original selector, ledger and queue matched their manifest
  hashes when checked after writing.
- A modified progress hash was rejected. An existing destination was rejected.
  On a case-insensitive directory full extraction was refused; the original
  evidence contains two pairs of paths that differ only in case.

- The registered source-object restorer initially verified 268 identities
  against both the shared repository and an independent clone, and refused
  an existing destination. After expansion, it independently restored 302
  identities representing 297 distinct Git blobs.
  A second restorer verified 118 additional identities, representing 114
  distinct hashes in that set and 91 hashes beyond the first set, against an
  independent clone.
- The transcript-derived evidence verifier checks the published JSON and
  note files against their manifests. Hash-matched records remain historical;
  normalized output fragments are explicitly distinguished from original
  file bytes.

- Four upstream source candidates were recreated from a fresh filtered
  SpacetimeDB clone at the pinned commit. The Git blob, source SHA-256 and
  transformed candidate SHA-256 all matched. The restorer refused an
  existing destination.

- A local archive of the committed recovery packet independently verified all
  included collections, including the redacted source records and draft
  evidence. It has not been published as of this local checkpoint.
- The local search-index builder indexed 27,671 overlapping records, returned
  expected keyword matches, and refused an existing destination. The index is
  a navigation aid, not audit evidence or a replacement for the ledger.

- Four final-frontier source identities and 387 recorded output lines were
  checked against the pinned revision using an independent clone. Their
  original registration bytes and audit authority remain unavailable.

These checks do not prove that a different machine has performed a restore,
that the deleted continuation can be reconstructed, or that any product
tests, audits or repairs have passed. No product source or database was
changed by this recovery.
