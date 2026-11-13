# Lossless transport for the first custody group

The preceding `INDEX.md` and `FIFTH-CUTOFF.md` describe the original first
custody packet. Its ciphertexts for objects 15 and 25 were larger than
GitHub's individual-file limit. A non-force push of local commit
`66b58f62a8bc57c2c272c1066faefc315d5d2232` was rejected for that reason;
the remote branch was observed still at parent `8b4b9b9aae3b9d9821418fa3ad5cff4a21e5384b`.
That rejected local commit is not an ancestor of this replacement checkpoint.
The original ciphertexts and the failed push's local records remain separately
held on the same machine.

`custody-encrypted/` now transports the unchanged original manifest, README
and verifier under `ORIGINAL-` names. Objects 15 and 25 are each divided into
two ordered pieces of at most 64 MiB; the remaining 44 objects are unchanged.
The split manifest binds each piece and the complete original ciphertext.
The streaming verifier checks all 46 original ciphertext identities and
within-group nonce uniqueness without a key. It does not authenticate or
decrypt them, and it does not repeat the earlier prior-inventory comparison.

Run `python3 -B custody-encrypted/VERIFY-SPLIT.py` from this directory for the
streaming check. To recover the unchanged original packet, run
`python3 -B custody-encrypted/RESTORE-SPLIT.py DESTINATION` with an absent
destination outside the transport whose parent already exists and whose
path has no symlink components. Concurrent path changes and unusual mount
aliases remain outside this same-machine check. The reconstructed packet contains
the original `MANIFEST.json`, `README.md`, `VERIFY.py` and 46 ciphertexts;
its verifier can then perform its documented no-key or key-assisted checks.
The original unsplit packet and a fresh same-machine reconstruction were
compared byte-for-byte and with recorded original full modes. This comparison is not an
independent backup or a new authenticated restore.

The split verifier's default mode checks the owner-executable state preserved
by Git; its strict mode checks full recorded transport modes on a suitable
same-machine copy. Git does not preserve every POSIX or directory mode. The
separate key, earlier ciphertexts, historical Git objects and private records
remain required for the corresponding recovery claims. Finite public scans do
not clear unknown secrets, encodings or opaque ciphertext; splitting also means
patterns spanning a piece boundary are not detected by a piece-only scan.
No recovered checkpoint 3821, independent custody, product validation or
semantic audit closure is implied.
