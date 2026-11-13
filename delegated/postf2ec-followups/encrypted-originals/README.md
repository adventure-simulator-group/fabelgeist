# Encrypted audit originals

Run `python3 -B VERIFY.py` to check ciphertext identities. Supplying the
separately held same-machine primary key authenticates the plaintext; a new,
nonexistent `--restore-root` restores without overwriting. Earlier q2812
anchors and nested packet keys remain dependencies described in their original
records; this collection does not add those keys.

This is not independent custody, complete history recovery, checkpoint 3821
recovery, product validation, or a claim that ciphertext was privacy-scanned.
