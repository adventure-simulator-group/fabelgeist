# Reconstructed q2354 source-only evidence

This is a path-redacted reconstruction of a separately held exact private
packet. The crosswalk records whole-file hashes, byte offsets and removed
lengths, with opaque aliases for repeated removed values. It deliberately
contains no removed-path digest. Public materials cannot recover removed bytes
or the private original on their own.

The preserved original manifest is `evidence/MANIFEST.json`, with SHA-256
`089ded8c977b60e88561f92f36716d93da82d5fb1fe3256364efa6eaa58ffd57`. Run `python3 verify-successor.py` to check
this reconstruction and manifest. Supply `--original-packet` for optional
private byte replay; add `--verify-anchors` and five separately held anchors
to rerun the original evidence-only verifier. No product code is executed.

The 95e age-eligibility sequence remains conditional; see
`evidence/data/ASSESSMENT.md` and `evidence/data/READ-LIMITS.json`. Original
reviewer attribution, older revision, missing checkpoint 3821, and bounded
scan and runtime limitations remain material.
