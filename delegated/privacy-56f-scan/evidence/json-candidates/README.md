# Draft reconstructed evidence

These five JSON files are review candidates, not published replacements.
Each original local account-home path value is replaced by a marker; all other
original bytes remain unchanged. `reconstruction-index.json` retains the
original commit, Git blob, SHA-256, field pointer and withheld value digest,
alongside each reconstructed file's independent SHA-256 and Git blob.

`verify-reconstruction.py` compares exact original Git bytes locally with the
five reconstructed candidates without printing any private values.
`verify-public.py` independently checks the candidates and bounded path patterns
without needing originals. Neither constitutes a general privacy, credential or
archive safety review. Exact-original historical verifiers require coordinated
changes; preserve the original identities and local custody before publication.
A later commit does not erase the already published historical commit.
