# Additive checkpoint 36 review correction

`CORRECTION.json` records one incorrect summary field in the earlier frozen
review and preserves its original hashes. The original byte-identity assertion
passed; a later reused variable caused the printed summary to name another 059
receipt. `verification-corrected.json` is the successful read-only rerun using
the corrected field expression. Only that summary field differs.

The corrected verifier accepts the same three explicit arguments as the original
review verifier: the local-only snapshot, the original 059 extraction, and the
publication directory. It does not run product code or mutate evidence.

`python3 verify-correction.py` verifies the additive packet itself. Supply the
original review directory as its optional argument to additionally compare both
verification outputs and prove the earlier review hashes remain unchanged.
