# Reconstructed q2340 source-only evidence

This is a path-redacted reconstruction of a separately held exact private
packet. The crosswalk records original and reconstructed whole-file hashes,
byte offsets and removed lengths, with opaque aliases for repeated removed
values. It deliberately contains no removed-path digest. Public materials
cannot recover removed bytes or the private original on their own.

The preserved original manifest is `evidence/MANIFEST.json`, with SHA-256
`f210c042497847286ab8f849487adf6ff45f91f0a2341c23afd443a679606d0c`. Run `python3 verify-successor.py` to check
the public reconstruction and manifest. For optional exact byte replay supply
`--original-packet` with a separately held private original. Add
`--verify-anchors` and all five named external anchors to invoke its original
evidence-only verifier. This does not run product code or validation.

The q2340 source finding remains conditional; see `evidence/data/ASSESSMENT.md`
and `evidence/data/READ-LIMITS.json`. Original reviewer attribution, older
revision, missing checkpoint 3821 and scan limits remain material. Historical
records and source snapshots are evidence, not current whole-lens closure.
