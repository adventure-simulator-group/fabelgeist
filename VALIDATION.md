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

- The source-object restorer verified 268 registered revision/path/hash
  identities representing 264 distinct Git blobs using both the shared Git
  repository and an independent clone. It refused an existing destination.
- The transcript-derived evidence verifier checks the published JSON and
  note files against their manifests. Hash-matched records remain historical;
  normalized output fragments are explicitly distinguished from original
  file bytes.

These checks do not prove that a different machine has performed a restore,
that the deleted continuation can be reconstructed, or that any product
tests, audits or repairs have passed. No product source or database was
changed by this recovery.
