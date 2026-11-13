# Encrypted evidence replay originals

Run `python3 -B VERIFY.py` to check ciphertext identity. The separately held
same-machine primary key authenticates and restores the exact transcript and
staged capture to a new destination. Use the sibling
`VERIFY-REPLAY-RECORD.py` against that restored collection for content checks.

Ciphertexts are opaque to public scans. This is not independent custody,
complete privacy, checkpoint 3821 recovery or product validation.
