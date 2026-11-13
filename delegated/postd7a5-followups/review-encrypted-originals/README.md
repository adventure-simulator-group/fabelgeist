# Encrypted review originals

Run `python3 -B VERIFY.py` to check ciphertext identities. Supplying the
separately held same-machine primary key authenticates the plaintext. A new,
nonexistent `--restore-root` restores it without overwriting. Earlier q2812
anchors and nested keys remain dependencies described in their records.

This does not establish independent custody, complete historical recovery,
checkpoint 3821 recovery, product behavior or ciphertext privacy.
