# Independent CP7 successor review

`REPORT.json` records the corrected reconstruction, exact retained contexts,
metadata differences, and privacy and custody limits. `EVIDENCE-IDENTITIES.json`
binds the original, prior review, earlier public archive, and successor.

The included `verify.py` reads evidence only. Set `CP7_ORIGINAL`,
`CP7_PRIOR_REVIEW`, and `CP7_EARLIER_STAGE` to their retained local locations.
The successor archive and both indices are copied into `evidence/`; the
private original and earlier frozen review remain external. The three
context-confirmed ordinary strings require exact, bounded allowlisting under
the earlier broad path matcher.
