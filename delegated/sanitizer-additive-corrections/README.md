# Additive sanitizer reconstruction

This evidence packet reconstructs 26 previously published or staged indexed
text members after restoring only proven non-path substitutions. It records
all 310 affected indexed member locations and their original, old-public, and
corrected SHA-256 identities in `RECONSTRUCTION.json`. One opaque checkpoint 7
archive is withheld: its original and old-public tar entry counts differ, and
ordinary text replacement cannot repair the tar structure. The original nested
archive and its unexamined descendants require separate tar-aware handling.

Thirteen reconstructed byte objects exactly match their independently screened
safe originals. Six additional gzip objects decode to original-identical,
screened-safe text but have newly compressed bytes. Seven other identities
retain all nonselected placeholders, including confirmed local-path captures
and a Windows-shaped capture retained conservatively. Original and public
source attribution, revision, and checkpoint identity remain separate. The
screen is bounded; it cannot rule out every possible sensitive value.

`PRIOR-SYSTEMIC-OCCURRENCES.json` and `PRIOR-SYSTEMIC-PROPOSAL.json` preserve
exact prior public records. `SCOPE-ADDENDUM.json` records their opaque-archive
limitation without rewriting those records. The original checkpoint 36 review
and its additive correction are preserved as two exact-byte directories; the
correction does not silently alter the earlier review. `CP35-CLARIFICATION.json`
records the separate attribution clarification.

Run `python3 verify-reconstruction.py` to verify this packet and its bounded
privacy scan without writing. Optional arguments select the original indexed
public archives and retained private mapping for independent byte checks; use
`python3 verify-reconstruction.py --help` for details. This is evidence
verification, not product testing or validation.
