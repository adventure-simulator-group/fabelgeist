# q2333 reconstructed source-evidence successor

This path-redacted reconstruction preserves the recovered 059 record, original
reviewer attribution, frozen and later revision qualifications, source-only
current assessment, open questions and read limits. It does not restore
checkpoint 3821, close the thirteen-lens audit, establish an alias or count, or
report a runtime pass. The substantive assessment is in `evidence/REPORT.md`.

The exact private original and original manifest remain separately held. The
original manifest SHA-256 is
`fbc3adf88a18e61e282ca2e4cd2557ce5f291761096626c703f519a33a1a09f6`.
`evidence/MANIFEST.json` preserves that historical manifest: its file hashes
refer to the private original, not the reconstructed files. The successor has
its own `SUCCESSOR-MANIFEST.json`.

`REDACTION-CROSSWALK.json` records original and reconstructed whole-file sizes
and hashes, original byte offsets, removed lengths and stable opaque aliases.
It does not publish a digest of any removed path value or span. The prior
successor was withheld because it did publish such digests; its privacy
qualification is retained here without the sensitive digest list. An opaque
alias does not reveal or publicly recover the removed value.

Run `python3 verify-successor.py` for public-only manifest, crosswalk and
reconstructed-file checks. With the separately held exact original, provide
`--original-packet` and its directory to check every original whole-file hash,
all substitutions and byte-for-byte replay. To check the commit/tree/blob
chains, historical queue slices and ledger snapshot as well, provide
`--verify-anchors` and the six external anchor arguments shown by
`python3 verify-successor.py --help`. The original and anchors may contain
private metadata and are neither included nor cleared for public release.

Public-only verification checks internal consistency; it does not demonstrate
possession of private originals, recover removed paths, or prove independent
custody, semantic truth, intended policy or runtime behavior. Scans are bounded
and do not clear unknown secrets, unsupported encodings and formats, symlinks,
Git history or external anchors. No product code, build, test, fixture, product
parser, macro, renderer or runtime probe was executed.
