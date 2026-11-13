# Bounded scan and candidate classification

The retained scan covers the earlier 139-file staged snapshot. It reported zero
known forbidden digest matches, four temporary-path candidates and no scan
exceptions. The four candidates occur in scanner regular-expression literals;
they are not observed local paths. The original scan output and an independent
classification are preserved without changing or suppressing those hits.

Run `python3 -B VERIFY.py --publication CHECKOUT` to check the four candidate
files. For the full independent review, first run `RESTORE-CAPTURE.py` with the
published checkout and a new output directory. Then run the verifier in
`independent-review/`, supplying the restored capture directory and the saved
`DIGEST-SCAN.json` as its receipt. The helper copies only captured, verified
bytes and refuses to overwrite an existing destination.

The restricted review and its replay are authenticated in
`encrypted-originals/`; a separately held same-machine key is required. These
checks do not inspect ciphertext or establish complete privacy, independent
custody, checkpoint 3821 recovery, product behavior or audit closure. The
139-file scan precedes this additional classification package; the final
staged snapshot receives its own bounded scan.
