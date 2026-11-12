# Reconstructed q2360–q2361 source-only evidence

This is a path-pattern-redacted reconstruction of a separately held exact
private packet. The crosswalk records whole-file hashes, byte offsets, removed
lengths and opaque equality aliases; it deliberately contains no individual
removed-value digest. One broad match is code-token text rather than a local
path and is redacted as well. Public materials cannot recover removed bytes or
the private original on their own.

The preserved private manifest is `evidence/MANIFEST.json`, SHA-256
`eea6cd826442eee69e22461f1e52a7a878c1bd96f59d8b806404bc72fb660dbf`. Run `python3 verify-successor.py` to check this reconstructed
manifest and crosswalk. Optional `--original-packet` verifies exact byte replay
against a separately held private original; `--verify-anchors` additionally
requires five named external anchors and invokes its evidence-only verifier.

The original e275 named document dispositions retain limited scope; no global
lens, native-binary, runtime or missing-checkpoint-3821 closure is implied.
See `evidence/data/ASSESSMENT.md` and `evidence/data/READ-LIMITS.json`.
