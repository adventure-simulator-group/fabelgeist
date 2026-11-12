# Additive evidence-verifier successor for two historical records

This packet derives a separately named successor from the prior frozen
three-object successor. The prior packet and the publication checkout are not
modified. Eight unique source replacements introduce an additional, independent
crosswalk for the writer output and the portability review, pin the three
corresponding historical manifests and qualify all three affected hash checks.
The earlier crosswalk and all unrelated assertions remain in place.

The original hash in the portability manifest identifies an already-redacted
historical review. Its separately recorded unredacted candidate hash remains a
historical assertion; those earlier bytes were not recovered here. The writer
transcript's JSON mode reports remain unchanged, but they do not establish that
the original checkpoint 3821 exists or has been recovered.

`sample-publication/` holds only the two reconstructed public records, their
three untouched historical manifests, the first crosswalk and the additional
crosswalk. Original private bytes are retained separately and are not included.
The next successor requires both crosswalks at the matching relative paths in a
complete publication checkout. It has not been run end to end.

Run `python3 -B verify-evidence.py` for bounded read-only checks. Optional
`--originals-root` verifies the two exact original-to-public byte substitutions;
`--publication-root` checks the observed historical objects, Git tree bindings
and untouched manifests; `--prior-stage` additionally checks the frozen first
successor and its three-object evidence helper. These checks execute only small
evidence helper declarations, not either complete verifier or product code.
`verify-privacy.py DIRECTORY` performs a bounded contextual path and
common-secret scan without printing candidate values. The previous aggregate
runner must be adapted to select one successor without also invoking either
older verifier. Existing Git history, archives, other exposed records and remote
copies are outside this packet's corrective scope.
