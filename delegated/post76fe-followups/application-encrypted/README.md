# Uninstalled application review custody

Run `python3 -B VERIFY.py` for ciphertext identity checks. Supply a separately
held primary key and a new `--restore-root` to authenticate and restore. The
embedded control contains the restricted source inventory and pre-encryption
nonce registration. Source roots and file and directory modes are preserved;
ownership and timestamps are normalized. The post-capture encryption,
restoration, replay, scan and delivery controls have a later cutoff and must
be preserved separately. Other Git, prior ciphertext and historical-anchor
inputs remain separate dependencies. Key custody is on the same machine, not
independent. This group does not establish checkpoint 3821 recovery, product
behavior, implementation or audit closure.
