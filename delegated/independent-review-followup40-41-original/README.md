# Independent follow-up transport review

`REPORT.json` records the result, prior reviewer involvement and limitations.
`CHECK.json`, `REPLAY.json` and `SCAN-RAW.json` contain the independently
recomputed evidence summaries. The four raw scan candidates are two regex
literals seen twice through tar representations; their classification is in the
report. `PATH-DIGEST-EXPOSURE.json` identifies public locations without including
either private cloth path value or its hash.

Run `python3 verify.py` to check this review packet. To repeat read-only input
checks, add `--stage` and `--private` with the separately retained stage and
restricted-original roots. Add `--scan` to repeat the longer bounded privacy
scan. The private original contents are never printed, copied into this review,
or written by the verifier. These checks do not run old packet verifiers or
product code.
