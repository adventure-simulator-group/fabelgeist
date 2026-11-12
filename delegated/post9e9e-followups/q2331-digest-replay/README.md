# q2331 removed-path digest replay

This is a bounded follow-up to q2331's redaction, not product validation.
`RESULT.json` and `RESULT-RECEIPTS.json` record the original check. `REPLAY.py`
uses only an externally supplied q2331 packet and separately held private key;
add `--receipts` for each separate receipt directory. The key is not published.
Without a key, it reports the digest comparison as not repeated. With a key it
authenticates all three originals, reconstructs substituted values only in
memory, checks exact removed values and their absolute directory prefixes in
raw and hex forms of SHA-256, SHA-1, and MD5, and prints counts and sanitized
filenames of any matches, never values or digest bytes. It does not check unknown
encodings or secrets and does not establish independent custody. Same-machine
key storage is a recovery dependency, not independent public recovery.
