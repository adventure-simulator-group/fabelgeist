# Encrypted post-freeze originals

Run `python3 -B VERIFY.py` to verify ciphertext identities. Supplying the
separately held same-machine primary key authenticates the archives, and a
new `--restore-root` restores them without overwriting. Earlier q2812 anchors
and other named inputs remain separate dependencies.

Ciphertext was not privacy-scanned. Same-machine recovery does not establish
independent custody, complete history, checkpoint 3821 or product behavior.
