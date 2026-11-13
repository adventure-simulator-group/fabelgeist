# Four scan-candidate locations

The preserved scan receipt reports four `unix-temp` candidates and zero
forbidden path-value digest hits. Each reported candidate is in a distinct
captured scanner source file. Each source contains one occurrence of the
relevant prefix, within a Python `re.compile` string literal; all four compiled
literals are equal. These are scanner-pattern syntax, not observed local path
values. No matched value or individual path-value digest is reproduced here.

All 139 captured staged files matched their recorded byte counts, SHA-256 and
Git blob identities. This is a classification of four recorded locations, not a
fresh complete privacy review, ciphertext inspection or independent custody.
The original four-hit receipt remains unchanged. Full replay needs that receipt
and the captured staged tree; private copies remain in same-machine custody.

`python3 VERIFY.py --capture CAPTURE_DIRECTORY --receipt RECEIPT_FILE` replays
these evidence-only checks without displaying candidate values. Omitting those
arguments checks only this public review packet.
