# Independent checkpoint 39 transport review

`REPORT.json` records the result and limitations. `CHECK-RESULTS.json` is the
non-sensitive output of the independent evidence-only input review;
`PUBLICATION-GIT-CHECK.json` records comparisons against locally retained Git
objects. `MANIFEST.json` binds these review files and the review programs.

Run `python3 verify.py` to check this review packet only. To repeat input checks,
pass `--full` with `--public`, `--local`, `--source2623`, `--source2625`,
`--source2626`, and `--prior` pointing to the corresponding separately retained
frozen directories. Pass `--publication` along with `--public` to repeat the
Git-object comparisons. The inputs are read only. The results capture file modes
and a local-account scan as observations on the review host, so exact replay of
those observations depends on the same input state and host. No old verifier or
product code is invoked.

The changed original objects are intentionally absent from this public-safe
review packet. They must be retained separately with appropriate access control;
the reviewed local-originals directory was observed to be readable by other
local users under its mode bits. The stage's commit-object metadata remains
visible, as disclosed in the review.
